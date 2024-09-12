use criterion::{black_box, Criterion, criterion_group, criterion_main};
use rand::Rng;
use cuckoo_rs::cuckoo::CuckooHashTable;

fn generate_random_numbers(count: usize) -> Vec<i32> {
    let mut rng = rand::thread_rng();
    (0..count).map(|_| rng.gen()).collect()
}

fn criterion_insert(c: &mut Criterion) {
    let numbers_1m = generate_random_numbers(1_000_000);
    c.bench_function("insert 1M elements", |b| {
        let mut table = CuckooHashTable::new();
        b.iter(|| {
            for &num in black_box(&numbers_1m) {
                table.insert(num);
            }
        })
    });
}

criterion_group!(benches, criterion_insert);
criterion_main!(benches);