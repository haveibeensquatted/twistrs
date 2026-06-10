use criterion::{criterion_group, criterion_main, Criterion};

use twistrs::{filter::Permissive, permutate::Domain};

fn bitsquatting(domain: &Domain) {
    domain.bitsquatting(&Permissive).for_each(drop)
}

fn homoglyph(domain: &Domain) {
    domain.homoglyph(&Permissive).for_each(drop)
}

fn hyphenation(domain: &Domain) {
    domain.hyphenation(&Permissive).for_each(drop)
}

fn insertion(domain: &Domain) {
    domain.insertion(&Permissive).for_each(drop)
}

fn omission(domain: &Domain) {
    domain.omission(&Permissive).for_each(drop)
}

fn repetition(domain: &Domain) {
    domain.repetition(&Permissive).for_each(drop)
}

fn replacement(domain: &Domain) {
    domain.replacement(&Permissive).for_each(drop)
}

fn subdomain(domain: &Domain) {
    domain.subdomain(&Permissive).for_each(drop)
}

fn transposition(domain: &Domain) {
    domain.transposition(&Permissive).for_each(drop)
}

fn vowel_swap(domain: &Domain) {
    domain.vowel_swap(&Permissive).for_each(drop)
}

fn keyword(domain: &Domain) {
    domain.keyword(&Permissive).for_each(drop)
}

fn tld(domain: &Domain) {
    domain.tld(&Permissive).for_each(drop)
}

fn criterion_benchmark(c: &mut Criterion) {
    let domain = Domain::new("example.com").unwrap();
    c.bench_function("bitsquatting example.com", |b| {
        b.iter(|| bitsquatting(&domain))
    });
    c.bench_function("homoglyph example.com", |b| b.iter(|| homoglyph(&domain)));
    c.bench_function("hyphenation example.com", |b| {
        b.iter(|| hyphenation(&domain))
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
