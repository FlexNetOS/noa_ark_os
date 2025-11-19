use criterion::{black_box, criterion_group, criterion_main, Criterion};
use noa_ui_api::schema::PageEnvelope;

fn bench_page_generation(c: &mut Criterion) {
    c.bench_function("page_envelope_with_sample", |b| {
        b.iter(|| {
            let page = PageEnvelope::with_sample(black_box("benchmark"));
            black_box(page)
        });
    });
}

criterion_group!(benches, bench_page_generation);
criterion_main!(benches);
