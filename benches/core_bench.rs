use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jigsaw::engine::mask::Mask;
use jigsaw::engine::rules::{Rule, RuleSet};
use std::str::FromStr;

fn benchmark_mask_iter(c: &mut Criterion) {
    let mask_str = "?l?d?d"; // 26 * 10 * 10 = 2600
    let mask = Mask::from_str(mask_str).unwrap();
    
    c.bench_function("mask_iter_2600", |b| {
        b.iter(|| {
            // consume iterator
            for item in mask.iter() {
                black_box(item);
            }
        })
    });
}

fn benchmark_mask_nth(c: &mut Criterion) {
    let mask_str = "?l?d?d";
    let mask = Mask::from_str(mask_str).unwrap();
    
    c.bench_function("mask_nth_candidate", |b| {
        b.iter(|| {
            black_box(mask.nth_candidate(black_box(1234)));
        })
    });
}

fn benchmark_rule_application(c: &mut Criterion) {
    // Reverse, Upper, Append '!'
    let rs = RuleSet::from_str("ru$!").unwrap();
    let mut candidate = b"password".to_vec();
    
    c.bench_function("rule_apply_ru$!", |b| {
        b.iter(|| {
            // We need to clone candidate every time or reset it, 
            // otherwise it keeps growing/changing.
            // Ideally we benchmark the apply operation on a fresh buffer.
            let mut buf = candidate.clone();
            rs.apply(&mut buf);
            black_box(buf);
        })
    });
}

criterion_group!(benches, benchmark_mask_iter, benchmark_mask_nth, benchmark_rule_application);
criterion_main!(benches);
