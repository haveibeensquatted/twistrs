use criterion::{criterion_group, criterion_main, Criterion};

use twistrs::permutate::Domain;

fn bitsquatting(domain: &Domain) {
    domain.bitsquatting().for_each(drop)
}

fn homoglyph(domain: &Domain) {
    domain.homoglyph().unwrap().for_each(drop)
}

fn hyphentation(domain: &Domain) {
    domain.hyphenation().for_each(drop)
}

fn insertion(domain: &Domain) {
    domain.insertion().for_each(drop)
}

fn omission(domain: &Domain) {
    domain.omission().for_each(drop)
}

fn repetition(domain: &Domain) {
    domain.repetition().for_each(drop)
}

fn replacement(domain: &Domain) {
    domain.replacement().for_each(drop)
}

fn subdomain(domain: &Domain) {
    domain.subdomain().for_each(drop)
}

fn transposition(domain: &Domain) {
    domain.transposition().for_each(drop)
}

fn vowel_swap(domain: &Domain) {
    domain.vowel_swap().for_each(drop)
}

fn keyword(domain: &Domain) {
    domain.keyword().for_each(drop)
}

fn tld(domain: &Domain) {
    domain.tld().for_each(drop)
}

fn criterion_benchmark(c: &mut Criterion) {
    let domain = Domain::new("example.com").unwrap();
    c.bench_function("bitsquatting example.com", |b| {
        b.iter(|| bitsquatting(&domain))
    });
    c.bench_function("homoglyph example.com", |b| b.iter(|| homoglyph(&domain)));
    c.bench_function("hyphentation example.com", |b| {
        b.iter(|| hyphentation(&domain))
    });
    c.bench_function("insertion example.com", |b| b.iter(|| insertion(&domain)));
    c.bench_function("omission example.com", |b| b.iter(|| omission(&domain)));
    c.bench_function("repetition example.com", |b| b.iter(|| repetition(&domain)));
    c.bench_function("replacement example.com", |b| {
        b.iter(|| replacement(&domain))
    });
    c.bench_function("subdomain example.com", |b| b.iter(|| subdomain(&domain)));
    c.bench_function("transposition example.com", |b| {
        b.iter(|| transposition(&domain))
    });
    c.bench_function("vowel_swap example.com", |b| b.iter(|| vowel_swap(&domain)));
    c.bench_function("keyword example.com", |b| b.iter(|| keyword(&domain)));
    c.bench_function("tld example.com", |b| b.iter(|| tld(&domain)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
