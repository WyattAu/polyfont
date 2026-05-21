use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use polyfont_core::*;

fn bench_scope_matches(c: &mut Criterion) {
    let mut group = c.benchmark_group("scope_matches");

    group.bench_function("exact_match", |b| {
        b.iter(|| {
            ScopeMatchEngine::scope_matches(
                black_box("keyword.control"),
                black_box("keyword.control"),
            )
        })
    });

    group.bench_function("hierarchical_match", |b| {
        b.iter(|| {
            ScopeMatchEngine::scope_matches(black_box("entity.name.function"), black_box("entity"))
        })
    });

    group.bench_function("deep_hierarchical_match", |b| {
        b.iter(|| {
            ScopeMatchEngine::scope_matches(
                black_box("entity.name.function.method"),
                black_box("entity.name"),
            )
        })
    });

    group.bench_function("wildcard_match", |b| {
        b.iter(|| ScopeMatchEngine::scope_matches(black_box("anything.at.all"), black_box("*")))
    });

    group.bench_function("no_match", |b| {
        b.iter(|| {
            ScopeMatchEngine::scope_matches(
                black_box("keyword.control"),
                black_box("string.quoted"),
            )
        })
    });

    group.finish();
}

fn bench_resolve_token(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_token");

    for rule_count in [10, 50, 100] {
        let rules = make_rules(rule_count);
        let engine = ScopeMatchEngine::from_rules(rules);

        let token = TokenInfo {
            text: "fn".to_string(),
            range: Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 2 },
            },
            scope: "scope.0.segment0".to_string(),
            modifiers: vec![],
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(rule_count),
            &token,
            |b, token| {
                b.iter(|| engine.resolve_token(black_box(token)));
            },
        );
    }

    group.finish();
}

fn bench_resolve_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_all");

    let tokens: Vec<TokenInfo> = (0..1000)
        .map(|i| TokenInfo {
            text: format!("token_{}", i),
            range: Range {
                start: Position {
                    line: i as u32,
                    column: 0,
                },
                end: Position {
                    line: i as u32,
                    column: 10,
                },
            },
            scope: format!("scope.{}.segment{}", i / 5, i % 5),
            modifiers: vec![],
        })
        .collect();

    for rule_count in [10, 50, 100] {
        let rules = make_rules(rule_count);

        group.bench_with_input(
            BenchmarkId::new("1000_tokens", rule_count),
            &tokens,
            |b, tokens| {
                let engine = ScopeMatchEngine::from_rules(rules.clone());
                b.iter(|| engine.resolve_all(black_box(tokens)));
            },
        );
    }

    group.finish();
}

fn bench_engine_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_construction");

    for rule_count in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(rule_count),
            &rule_count,
            |b, &rc| {
                b.iter_with_setup(
                    || make_rules(rc),
                    |rules| ScopeMatchEngine::from_rules(black_box(rules)),
                );
            },
        );
    }

    group.finish();
}

fn bench_specificity_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("specificity_sorting");

    for rule_count in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(rule_count),
            &rule_count,
            |b, &rc| {
                b.iter_with_setup(
                    || make_rules(rc),
                    |mut rules| {
                        rules.sort_by_key(|r| std::cmp::Reverse(r.specificity()));
                        black_box(&rules);
                    },
                );
            },
        );
    }

    group.finish();
}

fn make_rules(count: usize) -> Vec<FontRule> {
    (0..count)
        .map(|i| FontRule {
            scope: format!("scope.{}.segment{}", i / 5, i % 5),
            font: FontSpec::default_font("TestFont"),
        })
        .collect()
}

criterion_group!(
    benches,
    bench_scope_matches,
    bench_resolve_token,
    bench_resolve_all,
    bench_engine_construction,
    bench_specificity_sorting,
);
criterion_main!(benches);
