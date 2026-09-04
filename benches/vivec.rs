use divan::Bencher;
use fast_ds::vivec::ViVec;
use std::collections::LinkedList;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

const V: std::ops::Range<u128> = 0..(1 << 18);

#[divan::bench(sample_count = 500)]
fn collect_vivec() -> ViVec<u128> {
    V.collect()
}

#[divan::bench(sample_count = 500)]
fn collect_dll() -> LinkedList<u128> {
    V.collect()
}

#[divan::bench(sample_count = 500)]
fn sum_dll(bencher: Bencher) {
    let dll_data: LinkedList<_> = V.collect();
    bencher.bench(|| dll_data.iter().fold(0, |a, b| a + b));
}

#[divan::bench(sample_count = 500)]
fn sum_vivec_straight(bencher: Bencher) {
    let vivec_data: ViVec<_> = V.collect();
    bencher.bench(|| vivec_data.straight_iter().fold(0, |a, b| a + b));
}

#[divan::bench(sample_count = 500)]
fn sum_vivec_ordered_cached(bencher: Bencher) {
    let vivec_data: ViVec<_> = V.collect();
    bencher.bench(|| vivec_data.ordered_iter().fold(0, |a, b| a + b));
}

#[divan::bench(sample_count = 500)]
fn insert_vivec() -> ViVec<u128> {
    let mut data = ViVec::new();
    for i in V {
        data.append(i);
    }
    data
}

#[divan::bench(sample_count = 500)]
fn reinsert_vivec(bencher: Bencher) {
    let mut data = ViVec::new();
    for i in V {
        data.append(i);
    }
    for _ in V.skip(2) {
        data.pop();
    }
    bencher.bench_local(move || {
        for i in V {
            data.append(i);
        }
    });
}

#[divan::bench(sample_count = 500)]
fn insert_dll() -> LinkedList<u128> {
    let mut data = LinkedList::new();
    for i in V {
        data.push_front(i);
    }
    data
}
