use criterion::{criterion_group, criterion_main, Criterion};
use std::path::Path;
use ublox::*;

struct CpuProfiler;

impl criterion::profiler::Profiler for CpuProfiler {
    fn start_profiling(&mut self, benchmark_id: &str, _benchmark_dir: &Path) {
        cpuprofiler::PROFILER
            .lock()
            .unwrap()
            .start(format!("./{}.profile", benchmark_id).as_bytes())
            .unwrap();
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        cpuprofiler::PROFILER.lock().unwrap().stop().unwrap();
    }
}

fn profiled() -> Criterion {
    Criterion::default().with_profiler(CpuProfiler)
}

#[allow(dead_code)]
fn parse_all<T: UnderlyingBuffer>(mut parser: Parser<T>, data: &[u8], chunk_size: usize) -> usize {
    let mut count = 0;
    for chunk in data.chunks(chunk_size) {
        let mut it = parser.consume(chunk);
        loop {
            match it.next() {
                Some(Ok(_packet)) => {
                    count += 1;
                },
                Some(Err(e)) => {
                    panic!("No errors allowed! got: {:?}", e);
                },
                None => {
                    // We've eaten all the packets we have
                    break;
                },
            }
        }
    }
    count
}

pub fn criterion_benchmark(c: &mut Criterion) {
    for chunk in &[99, 100, 101, 256, 512, 1000, 1024] {
        c.bench_function(&format!("vec_parse_pos_{}", chunk), |b| {
            b.iter(|| {
                // TODO: requires pos.ubx file
                // let data = std::include_bytes!("pos.ubx");
                // let parser = Parser::default();
                // assert_eq!(parse_all(parser, data, *chunk), 2801);
                todo!()
            })
        });
    }
    for (buf_size, chunk) in &[(256, 100), (256, 256), (256, 512), (256, 1024)] {
        // let mut underlying = vec![0; *buf_size];
        c.bench_function(&format!("array_parse_pos_{}_{}", buf_size, chunk), |b| {
            b.iter(|| {
                // TODO: requires pos.ubx file
                // let data = std::include_bytes!("pos.ubx");
                // let underlying = FixedLinearBuffer::new(&mut underlying);
                // let parser = Parser::new(underlying);
                // assert_eq!(parse_all(parser, data, *chunk), 2801);
                todo!()
            })
        });
    }
}

criterion_group! {
name = benches;
config = profiled();
targets = criterion_benchmark
}
criterion_main!(benches);
