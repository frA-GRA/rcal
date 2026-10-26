use criterion::{Criterion, criterion_group, criterion_main};
use flate2::Compression;
use rcal::QName;
use rcal::calconfig::{CompressionType, SerializationFormat};
use rcal::externalizer::{
    CompressionExternalizer, XmlExternalizer, read_from_bytes, write_to_bytes,
};
use rcal::uci::CalMessage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct BenchMsg {
    id: String,
    value: f64,
    label: String,
    count: u32,
    active: bool,
    tags: Vec<String>,
    nested: BenchNested,
}

#[derive(Serialize, Deserialize, Clone)]
struct BenchNested {
    name: String,
    priority: i32,
    description: String,
}

impl CalMessage for BenchMsg {
    fn message_type_name() -> QName {
        QName::new(None, "BenchMsg")
    }
    fn cal_create() -> Self {
        BenchMsg {
            id: String::new(),
            value: 0.0,
            label: String::new(),
            count: 0,
            active: false,
            tags: Vec::new(),
            nested: BenchNested {
                name: String::new(),
                priority: 0,
                description: String::new(),
            },
        }
    }
}

fn make_msg() -> BenchMsg {
    BenchMsg {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        value: 3.14159,
        label: "benchmark-message".into(),
        count: 42,
        active: true,
        tags: vec!["alpha".into(), "beta".into(), "gamma".into()],
        nested: BenchNested {
            name: "subsystem-1".into(),
            priority: 5,
            description: "Primary subsystem component for benchmarking purposes".into(),
        },
    }
}

const ROOT: &str = "BenchMsg";

fn make_ext(ct: CompressionType) -> CompressionExternalizer {
    let xml = Box::new(XmlExternalizer::new(SerializationFormat::Xml));
    CompressionExternalizer::new(Some(xml), ct, Compression::default())
}

fn bench_gzip_write(c: &mut Criterion) {
    let ext = make_ext(CompressionType::Gzip);
    let msg = make_msg();
    c.bench_function("gzip_write", |b| {
        b.iter(|| write_to_bytes(&ext, criterion::black_box(&msg), ROOT).unwrap())
    });
}

fn bench_gzip_read(c: &mut Criterion) {
    let ext = make_ext(CompressionType::Gzip);
    let bytes = write_to_bytes(&ext, &make_msg(), ROOT).unwrap();
    c.bench_function("gzip_read", |b| {
        b.iter(|| read_from_bytes::<BenchMsg>(&ext, criterion::black_box(&bytes)).unwrap())
    });
}

fn bench_gzip_roundtrip(c: &mut Criterion) {
    let ext = make_ext(CompressionType::Gzip);
    let msg = make_msg();
    c.bench_function("gzip_roundtrip", |b| {
        b.iter(|| {
            let bytes = write_to_bytes(&ext, criterion::black_box(&msg), ROOT).unwrap();
            read_from_bytes::<BenchMsg>(&ext, &bytes).unwrap()
        })
    });
}

fn bench_deflate_write(c: &mut Criterion) {
    let ext = make_ext(CompressionType::Deflate);
    let msg = make_msg();
    c.bench_function("deflate_write", |b| {
        b.iter(|| write_to_bytes(&ext, criterion::black_box(&msg), ROOT).unwrap())
    });
}

fn bench_deflate_read(c: &mut Criterion) {
    let ext = make_ext(CompressionType::Deflate);
    let bytes = write_to_bytes(&ext, &make_msg(), ROOT).unwrap();
    c.bench_function("deflate_read", |b| {
        b.iter(|| read_from_bytes::<BenchMsg>(&ext, criterion::black_box(&bytes)).unwrap())
    });
}

fn bench_deflate_roundtrip(c: &mut Criterion) {
    let ext = make_ext(CompressionType::Deflate);
    let msg = make_msg();
    c.bench_function("deflate_roundtrip", |b| {
        b.iter(|| {
            let bytes = write_to_bytes(&ext, criterion::black_box(&msg), ROOT).unwrap();
            read_from_bytes::<BenchMsg>(&ext, &bytes).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_gzip_write,
    bench_gzip_read,
    bench_gzip_roundtrip,
    bench_deflate_write,
    bench_deflate_read,
    bench_deflate_roundtrip,
);
criterion_main!(benches);
