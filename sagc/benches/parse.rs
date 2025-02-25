// This file is used to benchmark the parser and the creation of the Grafana object from the parsed input.
// It measures the time it takes to parse the input and create the Grafana object.
// The input is the example.ssd file.
// The benchmark is run using the command `cargo bench --bench parse`

use criterion::{criterion_group, criterion_main, Criterion};
use sagc::grafana::Grafana;
use sagc::parser::parse_input;

fn criterion_benchmark_parse(c: &mut Criterion) {
    let input = include_str!("../grafana_example.ssd");  // Enter here the path to the desired ssd file

    // let mut group = c.benchmark_group("parse");
    // group.bench_with_input("parse", input, |b, input| b.iter(|| parse_input(input)));
    c.bench_function("parse", |b| b.iter(|| parse_input(input)));
    c.bench_function("create grafana", |b| {
        b.iter(|| {
            let sag = parse_input(input);
            match sag {
                Ok(sag) => {
                    let sag: Grafana = Grafana::from(sag);
                    sag
                }
                Err(e) => panic!("{:?}", e),
            }
        })
    });

    // group.finish();
}

criterion_group!(benches, criterion_benchmark_parse);
criterion_main!(benches);
