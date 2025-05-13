use criterion::{Criterion, PlottingBackend, criterion_group, criterion_main};
use mt_pir::benchmark::{BenchParams, bench};
use respire::{
    pir::{
        cuckoo_respire::CuckooRespireImpl, respire::RespireParamsExpanded,
        respire_harness::FactoryParams,
    },
    respire,
};
use std::time::Duration;

// Tree
const LEAF_EXP: usize = 12;
const N_RECORDS: usize = (1 << (LEAF_EXP + 1)) - 1;

// Respire config
const N_VEC: usize = 4;
const NU_1: usize = 5;
const NU_2: usize = 4;

// Cuckoo config
const BATCH_SIZE: usize = 12;
const N_BUCKETS: usize = 19;

const EXP_PARAMS: RespireParamsExpanded = FactoryParams::batch_32(BATCH_SIZE, N_VEC, NU_1, NU_2)
    .expand()
    .expand();

type Respire = respire!(EXP_PARAMS);
type CuckooRespire = CuckooRespireImpl<BATCH_SIZE, N_BUCKETS, N_RECORDS, Respire>;

fn criterion_benchmark(c: &mut Criterion) {
    let bench_params = BenchParams {
        leaf_exp: LEAF_EXP,
        batch_size: BATCH_SIZE,
        setup_t: 15,
        setup_n: 15,
        encode_t: 15,
        encode_n: 15,
        query_t: 10,
        query_n: 20,
        answer_t: 15,
        answer_n: 15,
        extract_t: 10,
        extract_n: 20,
    };
    bench::<CuckooRespire>(c, bench_params);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .plotting_backend(PlottingBackend::Plotters)
        .warm_up_time(Duration::from_secs(3));
    targets = criterion_benchmark
}
criterion_main!(benches);
