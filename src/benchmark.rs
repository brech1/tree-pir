//! Benchmark module

use bincode::{config::standard, serde::encode_to_vec};
use criterion::{BatchSize, Criterion, SamplingMode, black_box};
use lean_imt::lean_imt::LeanIMT;
use rand::{Rng, thread_rng};
use respire::pir::pir::{PIR, PIRRecordBytes};
use serde::Serialize;
use serde_json::from_slice;
use std::{
    fs::read,
    time::{Duration, Instant},
};

/// Zero record
const ZERO_RECORD: [u8; 32] = [0; 32];

/// Benchmark parameters
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
    fn setup(flat: Vec<[u8; 32]>, params: &BenchParams) -> Self {
        let label = params.label();
        let total = flat.len();
        let rec = Box::new(move |i: usize| {
            let raw = flat.get(i).map(|x| x.as_ref()).unwrap_or(&ZERO_RECORD);
            R::RecordBytes::from_bytes(raw).unwrap()
        });

        // Assert total records
        if params.batch_size == 1 {
            assert_eq!(total, R::NUM_RECORDS - 1);
        } else {
            assert_eq!(total, R::NUM_RECORDS);
        }

        println!("[{}] Initialization Report", label);
        println!("[{}] -----------------------", label);

        // Setup
        let start = Instant::now();
        let (qk, pp) = R::setup(None);
        let setup_ms = start.elapsed().as_secs_f64() * 1000.0;
        println!("[{}] Setup:             {:>8.2} ms", label, setup_ms);

        // Database encoding
        let start = Instant::now();
        let (db, hint) = R::encode_db(&rec, None);
        let encode_s = start.elapsed().as_secs_f64();
        println!("[{}] Encode DB:         {:>8.2} s", label, encode_s);

        let test_idxs: Vec<_> = (0..params.batch_size)
            .map(|_| thread_rng().gen_range(0..total))
            .collect();

        // Query generation
        let start = Instant::now();
        let (probe_q, s) = R::query(&qk, &test_idxs, &hint, None);
        let query_ms = start.elapsed().as_secs_f64() * 1000.0;
        println!("[{}] Query:             {:>8.2} ms", label, query_ms);

        // Answer computation
        let start = Instant::now();
        let probe_a = R::answer(&pp, &db, &probe_q, None, None);
        let answer_ms = start.elapsed().as_secs_f64() * 1000.0;
        println!("[{}] Answer:            {:>8.2} ms", label, answer_ms);

        // Extraction
        let start = Instant::now();
        let _ = R::extract(&qk, &probe_a, &s, None);
        let extract_us = start.elapsed().as_micros() as f64;
        println!("[{}] Extract:           {:>8.2} µs", label, extract_us);

        // Size section
        println!("[{}] -----------------------", label);

        // Client public parameters size
        let pp_bytes = encode_to_vec(&pp, standard()).unwrap().len();
        let pp_mb = pp_bytes as f64 / (1 << 20) as f64;
        println!(
            "[{}] Client public parameters size = {} bytes ({:.2} MB)",
            label, pp_bytes, pp_mb
        );

        // Client query size
        let q_bytes = encode_to_vec(&probe_q, standard()).unwrap().len();
        let q_kb = q_bytes as f64 / (1 << 10) as f64;
        println!(
            "[{}] Client query size = {} bytes ({:.2} KB)",
            label, q_bytes, q_kb
        );

        // Server answer size
        let a_bytes = encode_to_vec(&probe_a, standard()).unwrap().len();
        let a_kb = a_bytes as f64 / (1 << 10) as f64;
        println!(
            "[{}] Server answer size = {} bytes ({:.2} KB)",
            label, a_bytes, a_kb
        );

        println!("[{}] -----------------------", label);

        BenchContext {
            qk,
            pp,
            db,
            hint,
            total,
            rec,
        }
    }
}

/// Benchmark
pub fn bench<R: PIR>(c: &mut Criterion, bench_params: BenchParams)
where
    R::PublicParams: Serialize,
{
    let flat_tree = load_tree_flat(bench_params.leaf_exp);
    let context = BenchContext::<R>::setup(flat_tree, &bench_params);
    let label = bench_params.label();

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
