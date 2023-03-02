use criterion::{criterion_group, criterion_main, Criterion};
use rolldown_core::{BuildInputOptions, BundlerCore, InputItem};

#[tokio::main]
async fn threejs() {
  let mut bundler = BundlerCore::new(BuildInputOptions {
    input: vec![InputItem {
      name: "threejs".to_string(),
      import: "../three.js/src/Three.js".to_string(),
    }],
    ..Default::default()
  });
  // bundler
  //   .generate(rolldown_core::BuildOptions {
  //     ..Default::default()
  //   })
  //   .await
  //   .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("threejs", |b| b.iter(|| threejs));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
