use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use polyfont_config::PolyfontConfig;
use polyfont_core::ScopeMatchEngine;

fn generate_config_toml(rules: usize) -> String {
    let mut toml = String::from(
        "[config]\nversion = \"1.0\"\ndefault_font = { family = \"monospace\" }\n\n[[rules]]\nscope = \"source\"\nfont = { family = \"monospace\" }\n",
    );
    let scopes = vec![
        "keyword",
        "comment",
        "string",
        "entity.name.function",
        "entity.name.type",
        "variable",
        "constant",
        "punctuation",
        "operator",
        "storage.type",
    ];
    for i in 0..rules.saturating_sub(1) {
        let scope = scopes[i % scopes.len()];
        toml.push_str(&format!(
            "[[rules]]\nscope = \"{}.custom_{}\"\nfont = {{ family = \"TestFont{}\" }}\n",
            scope, i, i
        ));
    }
    toml
}

fn bench_config_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_parsing");

    for rule_count in [50, 100, 200, 500] {
        let toml = generate_config_toml(rule_count);

        group.bench_with_input(BenchmarkId::from_parameter(rule_count), &toml, |b, toml| {
            b.iter(|| black_box(toml::from_str::<PolyfontConfig>(black_box(toml))));
        });
    }

    group.finish();
}

fn bench_config_to_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_to_rules");

    for rule_count in [50, 100, 200, 500] {
        let toml = generate_config_toml(rule_count);
        let config: PolyfontConfig = toml::from_str(&toml).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(rule_count),
            &config,
            |b, config| {
                b.iter(|| black_box(config.to_rules()));
            },
        );
    }

    group.finish();
}

fn bench_engine_construction_from_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_construction_from_config");

    for rule_count in [50, 100, 200, 500] {
        let toml = generate_config_toml(rule_count);

        group.bench_with_input(BenchmarkId::from_parameter(rule_count), &toml, |b, toml| {
            b.iter_with_setup(
                || {
                    let config: PolyfontConfig = toml::from_str(black_box(toml)).unwrap();
                    config.to_rules()
                },
                |rules| {
                    black_box(ScopeMatchEngine::from_rules(rules));
                },
            );
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_config_parsing,
    bench_config_to_rules,
    bench_engine_construction_from_config,
);
criterion_main!(benches);
