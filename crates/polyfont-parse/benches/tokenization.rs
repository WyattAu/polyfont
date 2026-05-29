use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use polyfont_core::{FontRule, FontSpec};
use polyfont_parse::{OffsetEncoding, byte_offset_to_position};
use polyfont_scope::TrieScopeResolver;

fn generate_rust_document(lines: usize) -> String {
    let patterns = vec![
        "fn main() {",
        "    let x = 1;",
        "    println!(\"hello\");",
        "}",
        "// comment",
        "struct Foo;",
        "impl Bar for Foo {",
        "    fn baz(&self) -> bool {",
        "        true",
        "    }",
        "}",
        "pub enum Color {",
        "    Red,",
        "    Blue,",
        "}",
        "use std::io;",
        "mod tests;",
        "",
        "    # comment",
        "",
        "    if x > 0 {",
        "        return true;",
        "    }",
        "    false",
        "}",
        "const MAX: usize = 1000;",
    ];
    let mut doc = String::new();
    for i in 0..lines {
        doc.push_str(patterns[i % patterns.len()]);
        doc.push('\n');
    }
    doc
}

fn make_scope_rules(count: usize) -> Vec<FontRule> {
    (0..count)
        .map(|i| FontRule {
            scope: format!("scope.{}.segment{}", i / 5, i % 5),
            font: FontSpec::default_font("TestFont"),
        })
        .collect()
}

fn make_test_tokens(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("scope.{}.segment{}", i / 5, i % 5))
        .collect()
}

fn bench_naive_tokenization(c: &mut Criterion) {
    let mut group = c.benchmark_group("naive_tokenization");

    for line_count in [1_000, 10_000, 50_000] {
        let doc = generate_rust_document(line_count);
        let doc_len = doc.len();

        group.bench_with_input(BenchmarkId::new("lines", line_count), &doc, |b, doc| {
            b.iter(|| {
                let mut pos: usize = 0;
                let mut line: u32 = 0;
                let bytes = black_box(doc).as_bytes();
                for (i, &byte) in bytes.iter().enumerate() {
                    if byte == b'\n' {
                        line += 1;
                    }
                    pos = i;
                }
                black_box((pos, line));
            });
        });

        group.throughput(criterion::Throughput::Bytes(doc_len as u64));
    }

    group.finish();
}

fn bench_scope_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("scope_resolution");

    let tokens = make_test_tokens(1000);

    for rule_count in [50, 200, 500] {
        let trie = TrieScopeResolver::from_rules(make_scope_rules(rule_count));

        group.bench_with_input(
            BenchmarkId::new("trie", rule_count),
            &tokens,
            |b, tokens| {
                b.iter(|| {
                    for scope in tokens {
                        black_box(trie.resolve(black_box(scope)));
                    }
                });
            },
        );

        group.throughput(criterion::Throughput::Elements(tokens.len() as u64 * 3));
    }

    group.finish();
}

fn bench_byte_offset_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("byte_offset_conversion");

    for line_count in [1_000, 10_000, 50_000] {
        let doc = generate_rust_document(line_count);
        let doc_len = doc.len();
        let mid_offset = doc_len / 2;
        let quarter_offset = doc_len / 4;

        group.bench_with_input(
            BenchmarkId::new("utf8", line_count),
            &(doc.clone(), mid_offset),
            |b, (doc, offset)| {
                b.iter(|| black_box(byte_offset_to_position(doc, *offset, OffsetEncoding::Utf8)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("utf16", line_count),
            &(doc.clone(), mid_offset),
            |b, (doc, offset)| {
                b.iter(|| black_box(byte_offset_to_position(doc, *offset, OffsetEncoding::Utf16)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("utf8_quarter", line_count),
            &(doc.clone(), quarter_offset),
            |b, (doc, offset)| {
                b.iter(|| black_box(byte_offset_to_position(doc, *offset, OffsetEncoding::Utf8)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("utf16_quarter", line_count),
            &(doc.clone(), quarter_offset),
            |b, (doc, offset)| {
                b.iter(|| black_box(byte_offset_to_position(doc, *offset, OffsetEncoding::Utf16)));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_naive_tokenization,
    bench_scope_resolution,
    bench_byte_offset_conversion,
);
criterion_main!(benches);
