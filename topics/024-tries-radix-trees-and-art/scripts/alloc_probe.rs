//! Separate allocation telemetry. Never use its instrumented timings for selection.
//! Unsafe is confined to forwarding System's allocator contract unchanged.
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use topic024_tries_radix_trees_and_art::{Index, workload};
static ALLOC: AtomicUsize = AtomicUsize::new(0);
static REALLOC: AtomicUsize = AtomicUsize::new(0);
static FREE: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
struct Count;
// SAFETY: every operation forwards the exact pointer/layout to System.
unsafe impl GlobalAlloc for Count {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOC.fetch_add(1, Relaxed);
        BYTES.fetch_add(l.size(), Relaxed);
        // SAFETY: caller supplied a valid layout, passed through unchanged.
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        FREE.fetch_add(1, Relaxed);
        // SAFETY: caller supplies a live allocation and its original layout.
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        REALLOC.fetch_add(1, Relaxed);
        BYTES.fetch_add(n, Relaxed);
        // SAFETY: forwarding the original allocation and requested new size.
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static GLOBAL: Count = Count;
fn reset() {
    for x in [&ALLOC, &REALLOC, &FREE, &BYTES] {
        x.store(0, Relaxed);
    }
}
fn sample() -> [usize; 4] {
    [
        ALLOC.load(Relaxed),
        REALLOC.load(Relaxed),
        FREE.load(Relaxed),
        BYTES.load(Relaxed),
    ]
}
fn main() {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 3 {
        eprintln!("usage: alloc_probe CANDIDATE tiny|shared|wide");
        std::process::exit(2);
    }
    let (shape, n) = match a[2].as_str() {
        "tiny" => ("shared", 32),
        "wide" => ("wide", 8192),
        _ => ("shared", 8192),
    };
    let rows = workload(shape, n);
    reset();
    let index = Index::new(&a[1], &rows);
    let build = sample();
    reset();
    for i in 0..64 {
        black_box(index.get(black_box(&rows[i % rows.len()].0)));
    }
    let get = sample();
    reset();
    for i in 0..64 {
        let key = &rows[i % rows.len()].0;
        black_box(index.prefix(black_box(&key[..key.len() - 1])));
    }
    let prefix = sample();
    reset();
    drop(index);
    let teardown = sample();
    println!(
        "{{\"candidate\":\"{}\",\"shape\":\"{}\",\"fields\":[\"alloc\",\"realloc\",\"free\",\"requested_bytes\"],\"build\":{:?},\"get64\":{:?},\"prefix64\":{:?},\"teardown\":{:?}}}",
        a[1], a[2], build, get, prefix, teardown
    );
}
