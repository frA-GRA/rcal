use criterion::{Criterion, criterion_group, criterion_main};
use rcal::uci::base::{ContextV1, UUID, UuidTimestamp};

fn bench_uuid_generate_v4(c: &mut Criterion) {
    c.bench_function("uuid_generate_v4", |b| b.iter(UUID::generate_v4));
}

fn bench_uuid_generate_v1(c: &mut Criterion) {
    let ctx = ContextV1::new_random();
    let node: [u8; 6] = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB];

    c.bench_function("uuid_generate_v1", |b| {
        b.iter(|| {
            let ts = UuidTimestamp::now(&ctx);
            UUID::generate_v1(criterion::black_box(ts), criterion::black_box(&node))
        })
    });
}

fn bench_uuid_generate_default(c: &mut Criterion) {
    c.bench_function("uuid_generate_default", |b| {
        b.iter(|| UUID::generate(criterion::black_box(None)))
    });
}

fn bench_uuid_parse_str(c: &mut Criterion) {
    let s = "550e8400-e29b-41d4-a716-446655440000";
    c.bench_function("uuid_parse_str", |b| {
        b.iter(|| UUID::parse_str(criterion::black_box(s)).unwrap())
    });
}

criterion_group!(
    benches,
    bench_uuid_generate_v4,
    bench_uuid_generate_v1,
    bench_uuid_generate_default,
    bench_uuid_parse_str,
);
criterion_main!(benches);
