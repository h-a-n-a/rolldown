use criterion::{criterion_group, criterion_main, Criterion};
use rolldown_core::{Bundler, InputOptions};

#[tokio::main]
async fn threejs() {
  let mut bundler = Bundler::new(InputOptions {
    input: [(
      "threejs".to_string(),
      "../three.js/src/Three.js".to_string(),
    )]
    .into(),
    ..Default::default()
  });
  bundler
    .generate(rolldown_core::OutputOptions {
      ..Default::default()
    })
    .await
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("threejs", |b| b.iter(|| threejs()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
