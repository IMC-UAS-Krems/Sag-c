use criterion::{criterion_group, criterion_main, Criterion};
use sagc::grafana::Grafana;
use sagc::parser::parse_input;

fn criterion_benchmark_parse(c: &mut Criterion) {
    let input = include_str!("../../example.ssd");

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
                Err(e) => panic!("{}", e),
            }
        })
    });

    // group.finish();
}

criterion_group!(benches, criterion_benchmark_parse);
criterion_main!(benches);
