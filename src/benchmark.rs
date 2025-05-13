use bincode::{config::standard, serde::encode_to_vec};
use criterion::{BatchSize, Criterion, SamplingMode, black_box};
use lean_imt::lean_imt::LeanIMT;
use rand::{Rng, thread_rng};
use respire::pir::pir::{PIR, PIRRecordBytes};
use serde::Serialize;
use serde_json::from_slice;
use std::{fs::read, time::Duration};

/// Zero record
const ZERO_RECORD: [u8; 32] = [0; 32];

/// Benchmark parameters
#[derive(Clone, Copy)]
pub struct BenchParams {
    pub leaf_exp: usize,
    pub batch_size: usize,
    pub setup_t: u64,
    pub setup_n: usize,
    pub encode_t: u64,
    pub encode_n: usize,
    pub query_t: u64,
    pub query_n: usize,
    pub answer_t: u64,
    pub answer_n: usize,
    pub extract_t: u64,
    pub extract_n: usize,
}

impl BenchParams {
    pub fn label(&self) -> String {
        match self.batch_size {
            1 => format!("single-{}", self.leaf_exp),
            _ => format!("batch-{}", self.leaf_exp),
        }
    }
}

/// Benchmark context
pub struct BenchContext<R: PIR> {
    qk: R::QueryKey,
    pp: R::PublicParams,
    db: R::Database,
    hint: R::DatabaseHint,
    total: usize,
    rec: Box<dyn Fn(usize) -> R::RecordBytes>,
}

impl<R: PIR> BenchContext<R>
where
    R::PublicParams: Serialize,
{
    fn new(flat: Vec<[u8; 32]>) -> Self {
        let total = flat.len();
        let rec = Box::new(move |i: usize| {
            let raw = flat.get(i).map(|x| x.as_ref()).unwrap_or(&ZERO_RECORD);
            R::RecordBytes::from_bytes(raw).unwrap()
        });

        let (qk, pp) = R::setup(None);
        let (db, hint) = R::encode_db(&rec, None);

        BenchContext {
            qk,
            pp,
            db,
            hint,
            total,
            rec,
        }
    }

    fn report_sizes(&self, params: &BenchParams) {
        let pp_bytes = encode_to_vec(&self.pp, standard()).unwrap().len();
        println!(
            "[{}] Client public parameters size = {} bytes ({:.2} MB)",
            params.label(),
            pp_bytes,
            (pp_bytes as f64) / (1 << 20) as f64
        );

        let probe_idxs: Vec<_> = (0..params.batch_size)
            .map(|_| thread_rng().gen_range(0..self.total))
            .collect();
        let (probe_q, _s) = R::query(&self.qk, &probe_idxs, &self.hint, None);
        let q_bytes = encode_to_vec(&probe_q, standard()).unwrap().len();
        println!(
            "[{}] Client query size = {} bytes ({:.2} KB)",
            params.label(),
            q_bytes,
            (q_bytes as f64) / (1 << 10) as f64
        );

        let probe_a = R::answer(&self.pp, &self.db, &probe_q, None, None);
        let a_bytes = encode_to_vec(&probe_a, standard()).unwrap().len();
        println!(
            "[{}] Server answer size = {} bytes ({:.2} KB)",
            params.label(),
            a_bytes,
            (a_bytes as f64) / (1 << 10) as f64
        );
    }
}

/// Benchmark
pub fn bench<R: PIR>(c: &mut Criterion, bench_params: BenchParams)
where
    R::PublicParams: Serialize,
{
    let flat = load_tree_flat(bench_params.leaf_exp);
    let context = BenchContext::<R>::new(flat);
    let label = bench_params.label();
    context.report_sizes(&bench_params);

    // Setup
    {
        let mut g = c.benchmark_group(format!("{}-setup", label));
        g.measurement_time(Duration::from_secs(bench_params.setup_t))
            .sample_size(bench_params.setup_n)
            .sampling_mode(SamplingMode::Flat);
        g.bench_function("setup", |b| {
            b.iter_batched(
                || (),
                |_| {
                    let (_k, _pp) = R::setup(None);
                },
                BatchSize::SmallInput,
            )
        });
        g.finish();
    }

    // Database encoding
    {
        let mut g = c.benchmark_group(format!("{}-encode", label));
        g.measurement_time(Duration::from_secs(bench_params.encode_t))
            .sample_size(bench_params.encode_n)
            .sampling_mode(SamplingMode::Flat);
        g.bench_function("encode_db", |b| {
            b.iter_batched(
                || (),
                |_| {
                    R::encode_db(&context.rec, None);
                },
                BatchSize::SmallInput,
            )
        });
        g.finish();
    }

    // Query generation
    {
        let mut g = c.benchmark_group(format!("{}-query", label));
        g.measurement_time(Duration::from_secs(bench_params.query_t))
            .sample_size(bench_params.query_n);
        g.bench_function("query", |b| {
            b.iter_batched(
                || {
                    (0..bench_params.batch_size)
                        .map(|_| thread_rng().gen_range(0..context.total))
                        .collect::<Vec<_>>()
                },
                |idxs| black_box(R::query(&context.qk, &idxs, &context.hint, None)),
                BatchSize::SmallInput,
            )
        });
        g.finish();
    }

    // Answer
    {
        let mut g = c.benchmark_group(format!("{}-answer", label));
        g.measurement_time(Duration::from_secs(bench_params.answer_t))
            .sample_size(bench_params.answer_n)
            .sampling_mode(SamplingMode::Flat);
        g.bench_function("answer", |b| {
            b.iter_batched(
                || {
                    let idxs = (0..bench_params.batch_size)
                        .map(|_| thread_rng().gen_range(0..context.total))
                        .collect::<Vec<_>>();
                    let (q, _s) = R::query(&context.qk, &idxs, &context.hint, None);
                    q
                },
                |q| black_box(R::answer(&context.pp, &context.db, &q, None, None)),
                BatchSize::SmallInput,
            )
        });
        g.finish();
    }

    // Extract
    {
        let mut g = c.benchmark_group(format!("{}-extract", label));
        g.measurement_time(Duration::from_secs(bench_params.extract_t))
            .sample_size(bench_params.extract_n);
        g.bench_function("extract", |b| {
            b.iter_batched(
                || {
                    let idxs = (0..bench_params.batch_size)
                        .map(|_| thread_rng().gen_range(0..context.total))
                        .collect::<Vec<_>>();
                    let (q, s) = R::query(&context.qk, &idxs, &context.hint, None);
                    let a = R::answer(&context.pp, &context.db, &q, None, None);
                    (s, a)
                },
                |(s, a)| black_box(R::extract(&context.qk, &a, &s, None)),
                BatchSize::SmallInput,
            )
        });
        g.finish();
    }
}

/// Loads and flattens LeanIMT
pub fn load_tree_flat(leaf_exp: usize) -> Vec<[u8; 32]> {
    let path = format!("./trees/tree_{leaf_exp}.json");
    let data = read(&path).unwrap();
    let tree: LeanIMT<32> = from_slice(&data).unwrap();
    tree.nodes().iter().flatten().copied().collect()
}
