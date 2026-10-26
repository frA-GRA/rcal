use criterion::{Criterion, criterion_group, criterion_main};
use rcal::exi;

// Small: single element, two attributes, character data (~120 bytes)
const SMALL_XML: &[u8] = br#"<?xml version="1.0"?><SystemStatus messageID="abc" version="1"><State>Active</State></SystemStatus>"#;

// Medium: nested elements, many attributes (~750 bytes)
const MEDIUM_XML: &[u8] = br#"<?xml version="1.0"?>
<SystemStatus messageID="abc123" version="2.5" priority="5">
  <Header senderID="SYS-001" receiverID="SYS-002" sequenceNumber="42">
    <Timestamp>2026-10-26T12:00:00Z</Timestamp>
    <SecurityToken>token-value-here</SecurityToken>
  </Header>
  <State>Active</State>
  <Components>
    <Component id="comp-1" type="Sensor" status="Online" health="100">
      <Description>Primary sensor array</Description>
      <Metrics latency="12" throughput="1000" errorRate="0"/>
    </Component>
    <Component id="comp-2" type="Actuator" status="Standby" health="98">
      <Description>Secondary actuator bank</Description>
      <Metrics latency="8" throughput="500" errorRate="0"/>
    </Component>
  </Components>
  <Diagnostics level="INFO">All systems nominal</Diagnostics>
</SystemStatus>"#;

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("exi_encode");

    group.bench_function("small", |b| {
        b.iter(|| exi::encode(criterion::black_box(SMALL_XML)).unwrap())
    });

    group.bench_function("medium", |b| {
        b.iter(|| exi::encode(criterion::black_box(MEDIUM_XML)).unwrap())
    });

    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let small_exi = exi::encode(SMALL_XML).unwrap();
    let medium_exi = exi::encode(MEDIUM_XML).unwrap();

    let mut group = c.benchmark_group("exi_decode");

    group.bench_function("small", |b| {
        b.iter(|| exi::decode(criterion::black_box(&small_exi)).unwrap())
    });

    group.bench_function("medium", |b| {
        b.iter(|| exi::decode(criterion::black_box(&medium_exi)).unwrap())
    });

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("exi_roundtrip");

    group.bench_function("small", |b| {
        b.iter(|| {
            let encoded = exi::encode(criterion::black_box(SMALL_XML)).unwrap();
            exi::decode(&encoded).unwrap()
        })
    });

    group.bench_function("medium", |b| {
        b.iter(|| {
            let encoded = exi::encode(criterion::black_box(MEDIUM_XML)).unwrap();
            exi::decode(&encoded).unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, bench_encode, bench_decode, bench_roundtrip);
criterion_main!(benches);
