use criterion::{Criterion, PlottingBackend, criterion_group, criterion_main};
use respire::{
    pir::{
        cuckoo_respire::CuckooRespireImpl, respire::RespireParamsExpanded,
        respire_harness::FactoryParams,
    },
    respire,
};
use tree_pir::benchmark::{BenchParams, bench};

// Tree
const LEAF_EXP: usize = 16;
const N_RECORDS: usize = 131_071;

// Respire config
const N_VEC: usize = 4;
const NU_1: usize = 6;
const NU_2: usize = 5;

// Cuckoo config
const BATCH_SIZE: usize = 16;
const N_BUCKETS: usize = 28;

const EXP_PARAMS: RespireParamsExpanded = FactoryParams::batch_32(BATCH_SIZE, N_VEC, NU_1, NU_2)
    .expand()
    .expand();

type Respire = respire!(EXP_PARAMS);
type CuckooRespire = CuckooRespireImpl<BATCH_SIZE, N_BUCKETS, N_RECORDS, Respire>;

fn criterion_benchmark(c: &mut Criterion) {
    let bench_params = BenchParams {
        leaf_exp: LEAF_EXP,
        batch_size: BATCH_SIZE,
        setup_t: 10,
        setup_n: 20,
        encode_t: 80,
        encode_n: 10,
        query_t: 10,
        query_n: 20,
        answer_t: 100,
        answer_n: 10,
        extract_t: 100,
        extract_n: 10,
    };
    bench::<CuckooRespire>(c, bench_params);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .plotting_backend(PlottingBackend::Plotters);
    targets = criterion_benchmark
}
criterion_main!(benches);
