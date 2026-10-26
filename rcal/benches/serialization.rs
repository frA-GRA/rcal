use criterion::{Criterion, criterion_group, criterion_main};
use rcal::QName;
use rcal::calconfig::SerializationFormat;
use rcal::externalizer::{XmlExternalizer, read_from_bytes, write_to_bytes};
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
        value: std::f64::consts::PI,
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

fn bench_xml_write(c: &mut Criterion) {
    let ext = XmlExternalizer::new(SerializationFormat::Xml);
    let msg = make_msg();

    c.bench_function("xml_write", |b| {
        b.iter(|| write_to_bytes(&ext, criterion::black_box(&msg), ROOT).unwrap())
    });
}

fn bench_xml_read(c: &mut Criterion) {
    let ext = XmlExternalizer::new(SerializationFormat::Xml);
    let bytes = write_to_bytes(&ext, &make_msg(), ROOT).unwrap();

    c.bench_function("xml_read", |b| {
        b.iter(|| read_from_bytes::<BenchMsg>(&ext, criterion::black_box(&bytes)).unwrap())
    });
}

fn bench_xml_roundtrip(c: &mut Criterion) {
    let ext = XmlExternalizer::new(SerializationFormat::Xml);
    let msg = make_msg();

    c.bench_function("xml_roundtrip", |b| {
        b.iter(|| {
            let bytes = write_to_bytes(&ext, criterion::black_box(&msg), ROOT).unwrap();
            read_from_bytes::<BenchMsg>(&ext, &bytes).unwrap()
        })
    });
}

#[cfg(feature = "exi")]
fn bench_exi_write(c: &mut Criterion) {
    use rcal::externalizer::new_exi_externalizer;

    let xml = Box::new(XmlExternalizer::new(SerializationFormat::Xml));
    let ext = new_exi_externalizer(xml);
    let msg = make_msg();

    c.bench_function("exi_write", |b| {
        b.iter(|| write_to_bytes(&ext, criterion::black_box(&msg), ROOT).unwrap())
    });
}

#[cfg(feature = "exi")]
fn bench_exi_read(c: &mut Criterion) {
    use rcal::externalizer::new_exi_externalizer;

    let xml = Box::new(XmlExternalizer::new(SerializationFormat::Xml));
    let ext = new_exi_externalizer(xml);
    let bytes = write_to_bytes(&ext, &make_msg(), ROOT).unwrap();

    c.bench_function("exi_read", |b| {
        b.iter(|| read_from_bytes::<BenchMsg>(&ext, criterion::black_box(&bytes)).unwrap())
    });
}

#[cfg(feature = "exi")]
fn bench_exi_roundtrip(c: &mut Criterion) {
    use rcal::externalizer::new_exi_externalizer;

    let xml = Box::new(XmlExternalizer::new(SerializationFormat::Xml));
    let ext = new_exi_externalizer(xml);
    let msg = make_msg();

    c.bench_function("exi_roundtrip", |b| {
        b.iter(|| {
            let bytes = write_to_bytes(&ext, criterion::black_box(&msg), ROOT).unwrap();
            read_from_bytes::<BenchMsg>(&ext, &bytes).unwrap()
        })
    });
}

#[cfg(not(feature = "exi"))]
criterion_group!(
    benches,
    bench_xml_write,
    bench_xml_read,
    bench_xml_roundtrip
);

#[cfg(feature = "exi")]
criterion_group!(
    benches,
    bench_xml_write,
    bench_xml_read,
    bench_xml_roundtrip,
    bench_exi_write,
    bench_exi_read,
    bench_exi_roundtrip
);

criterion_main!(benches);
