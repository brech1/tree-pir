use criterion::{Criterion, PlottingBackend, criterion_group, criterion_main};
use mt_pir::benchmark::{BenchParams, bench};
use respire::{
    pir::{respire::RespireParamsExpanded, respire_harness::FactoryParams},
    respire,
};
use std::time::Duration;

// Tree
const LEAF_EXP: usize = 16;

// Respire config
const NU_1: usize = 9;
const NU_2: usize = 7;

const EXP_PARAMS: RespireParamsExpanded = FactoryParams::single_record_32(NU_1, NU_2)
    .expand()
    .expand();
type Respire = respire!(EXP_PARAMS);

fn criterion_benchmark(c: &mut Criterion) {
    let bench_params = BenchParams {
        leaf_exp: LEAF_EXP,
        batch_size: 1,
        setup_t: 30,
        setup_n: 10,
        encode_t: 30,
        encode_n: 10,
        query_t: 15,
        query_n: 15,
        answer_t: 30,
        answer_n: 10,
        extract_t: 15,
        extract_n: 15,
    };
    bench::<Respire>(c, bench_params);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .plotting_backend(PlottingBackend::Plotters)
        .warm_up_time(Duration::from_secs(3));
    targets = criterion_benchmark
}
criterion_main!(benches);
