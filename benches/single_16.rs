use criterion::{Criterion, PlottingBackend, criterion_group, criterion_main};
use respire::{
    pir::{respire::RespireParamsExpanded, respire_harness::FactoryParams},
    respire,
};
use tree_pir::benchmark::{BenchParams, bench};

// Tree
const LEAF_EXP: usize = 16;

// Respire config
const NU_1: usize = 7;
const NU_2: usize = 6;

const EXP_PARAMS: RespireParamsExpanded = FactoryParams::single_record_32(NU_1, NU_2)
    .expand()
    .expand();
type Respire = respire!(EXP_PARAMS);

fn criterion_benchmark(c: &mut Criterion) {
    let bench_params = BenchParams {
        leaf_exp: LEAF_EXP,
        batch_size: 1,
        setup_t: 10,
        setup_n: 20,
        encode_t: 70,
        encode_n: 10,
        query_t: 10,
        query_n: 20,
        answer_t: 10,
        answer_n: 10,
        extract_t: 10,
        extract_n: 10,
    };
    bench::<Respire>(c, bench_params);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .plotting_backend(PlottingBackend::Plotters);
    targets = criterion_benchmark
}
criterion_main!(benches);
