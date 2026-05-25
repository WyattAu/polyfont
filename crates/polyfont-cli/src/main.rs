use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use polyfont_config::{ConfigLoader, PolyfontConfig};
use polyfont_core::{FontStyle, FontWeight};
use polyfont_fonts::{FontScanner, create_discovery};
use polyfont_scope::ScopeResolver;
use polyfont_themes::{ThemeExporter, ThemeRegistry};

#[derive(Parser)]
#[command(name = "polyfont")]
#[command(about = "CLI tool for polyfont multi-font configuration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Check,
    Vscode {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Neovim,
    Kitty,
    Dump,
    Font {
        #[command(subcommand)]
        action: FontAction,
    },
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
}

#[derive(Subcommand)]
enum FontAction {
    List,
    Check {
        family: Vec<String>,
    },
    /// Download and install a font from a known source.
    Install {
        family: Vec<String>,
    },
    /// Show a preview of a font family as SVG.
    Preview {
        family: String,
        #[arg(
            short,
            long,
            default_value = "The quick brown fox jumps over the lazy dog. 0123456789 {}[]()"
        )]
        text: String,
    },
}

#[derive(Subcommand)]
enum ThemeAction {
    List,
    Show {
        name: String,
    },
    Apply {
        name: String,
    },
    /// Import a VSCode JSON or TextMate .tmTheme file.
    Import {
        /// Path to the theme file (.json or .tmTheme).
        path: PathBuf,
        /// Font mapping TOML file (optional, uses defaults).
        #[arg(short, long)]
        mapping: Option<PathBuf>,
    },
    /// Export current config as a theme file.
    Export {
        /// Output format: toml or vscode.
        #[arg(short, long, default_value = "toml")]
        format: String,
        /// Output file path (stdout if omitted).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn load_config(path: Option<&PathBuf>) -> Result<PolyfontConfig> {
    if let Some(p) = path {
        ConfigLoader::load_from_path(p)
            .with_context(|| format!("failed to load config from {}", p.display()))
    } else {
        let dir = std::env::current_dir().context("failed to get current directory")?;
        ConfigLoader::load_from_dir(&dir)
            .with_context(|| "no .polyfont.toml found in current directory or ancestors")
    }
}

const fn weight_label(w: FontWeight) -> &'static str {
    match w {
        FontWeight::Thin => "thin",
        FontWeight::ExtraLight => "extra-light",
        FontWeight::Light => "light",
        FontWeight::Regular => "regular",
        FontWeight::Medium => "medium",
        FontWeight::SemiBold => "semi-bold",
        FontWeight::Bold => "bold",
        FontWeight::ExtraBold => "extra-bold",
        FontWeight::Black => "black",
    }
}

const fn style_label(s: FontStyle) -> &'static str {
    match s {
        FontStyle::Normal => "normal",
        FontStyle::Italic => "italic",
        FontStyle::Oblique => "oblique",
    }
}

const fn weight_css(w: FontWeight) -> &'static str {
    match w {
        FontWeight::Thin => "100",
        FontWeight::ExtraLight => "200",
        FontWeight::Light => "300",
        FontWeight::Regular => "normal",
        FontWeight::Medium => "500",
        FontWeight::SemiBold => "600",
        FontWeight::Bold => "bold",
        FontWeight::ExtraBold => "800",
        FontWeight::Black => "900",
    }
}

fn font_family_with_fallbacks(family: &str, fallbacks: &[String]) -> String {
    let mut parts = vec![family.to_string()];
    parts.extend(fallbacks.iter().cloned());
    parts.join(", ")
}

fn cmd_check(config: &PolyfontConfig) {
    println!("Config version: {}", config.version);

    let mut all_families: Vec<String> = Vec::new();

    if let Some(ref default) = config.default {
        all_families.push(default.family.clone());
        all_families.extend(default.fallbacks.iter().cloned());
        println!(
            "\nDefault font: {} (weight: {}, style: {})",
            default.family,
            weight_label(default.weight),
            style_label(default.style),
        );
        if !default.fallbacks.is_empty() {
            println!("  Fallbacks: {}", default.fallbacks.join(", "));
        }
    }

    println!("\nRules ({}):", config.rules.len());
    for (i, rule) in config.rules.iter().enumerate() {
        all_families.push(rule.font.family.clone());
        all_families.extend(rule.font.fallbacks.iter().cloned());
        println!(
            "  [{}] scope: {} -> {} (weight: {}, style: {})",
            i,
            rule.scope,
            rule.font.family,
            weight_label(rule.font.weight),
            style_label(rule.font.style),
        );
        if !rule.font.fallbacks.is_empty() {
            println!("       fallbacks: {}", rule.font.fallbacks.join(", "));
        }
    }

    all_families.sort();
    all_families.dedup();

    println!("\nFont availability:");
    let scanner = FontScanner::new();
    let mut missing = Vec::new();
    for family in &all_families {
        if scanner.is_available(family) {
            println!("  [ok] {family}");
        } else {
            println!("  [missing] {family} (install with: polyfont font install \"{family}\")");
            missing.push(family.clone());
        }
    }

    if missing.is_empty() {
        println!("\nAll fonts found. Config is valid.");
    } else {
        println!(
            "\nWarning: {} font(s) not found in system font directories.",
            missing.len()
        );
        println!("This may indicate a typo or the font is installed in a non-standard location.");
    }
}

fn cmd_vscode(config: &PolyfontConfig, output: Option<&PathBuf>) -> Result<()> {
    let mut rules_json: Vec<serde_json::Value> = Vec::new();

    for rule in &config.rules {
        let mut settings = serde_json::Map::new();
        settings.insert(
            "fontFamily".into(),
            serde_json::Value::String(font_family_with_fallbacks(
                &rule.font.family,
                &rule.font.fallbacks,
            )),
        );
        if rule.font.weight != FontWeight::Regular {
            settings.insert(
                "fontWeight".into(),
                serde_json::Value::String(weight_css(rule.font.weight).to_string()),
            );
        }
        if rule.font.style != FontStyle::Normal {
            settings.insert(
                "fontStyle".into(),
                serde_json::Value::String(style_label(rule.font.style).to_string()),
            );
        }
        rules_json.push(serde_json::json!({
            "scope": rule.scope,
            "settings": serde_json::Value::Object(settings),
        }));
    }

    let snippet = serde_json::json!({
        "editor.tokenColorCustomizations": {
            "textMateRules": rules_json,
        }
    });

    match output {
        Some(output_path) => {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create directory {}", parent.display()))?;
            }

            let doc = if output_path.exists() {
                let content = std::fs::read_to_string(output_path)
                    .with_context(|| format!("failed to read {}", output_path.display()))?;
                let mut val: serde_json::Value = serde_json::from_str(&content)
                    .with_context(|| format!("failed to parse {}", output_path.display()))?;
                let obj = val
                    .as_object_mut()
                    .context("existing settings.json is not an object")?;
                let snippet_obj = snippet.as_object().unwrap();
                for (k, v) in snippet_obj {
                    obj.insert(k.clone(), v.clone());
                }
                val
            } else {
                snippet
            };

            let json = serde_json::to_string_pretty(&doc)?;
            std::fs::write(output_path, json)
                .with_context(|| format!("failed to write {}", output_path.display()))?;
            eprintln!("Written to {}", output_path.display());
        }
        None => {
            println!("{}", serde_json::to_string_pretty(&snippet)?);
        }
    }

    Ok(())
}

fn cmd_neovim(config: &PolyfontConfig) {
    println!("-- polyfont neovim configuration");
    println!("-- Generated by polyfont-cli");

    if let Some(ref default) = config.default {
        println!("\n-- Default font: {}", default.family);
        println!("vim.o.guifont = '{}'", default.family);
    }

    println!("\n-- Scope-specific font rules:");

    for rule in &config.rules {
        let scope_var = rule.scope.replace('.', "_");
        let weight = weight_label(rule.font.weight);
        let style = style_label(rule.font.style);

        let mut font_parts = vec![rule.font.family.clone()];
        if rule.font.weight != FontWeight::Regular {
            font_parts.push(format!("h:{weight}"));
        }
        if rule.font.style != FontStyle::Normal {
            font_parts.push(style.to_string());
        }
        let font_spec = font_parts.join(":");

        println!();
        println!(
            "-- {}: {} ({}, {})",
            rule.scope, rule.font.family, weight, style
        );
        println!("vim.api.nvim_set_hl(0, '@{scope_var}', {{ font = '{font_spec}' }})");
    }

    println!();
}

fn cmd_kitty(config: &PolyfontConfig) {
    println!("# polyfont kitty configuration");
    println!("# Generated by polyfont-cli");
    println!("#");
    println!("# Note: Kitty uses symbol_map for unicode ranges, not TextMate scopes.");
    println!("# Adjust the unicode ranges below to match your needs.");

    if let Some(ref default) = config.default {
        println!("\n# Default font: {}", default.family);
    }

    println!();

    for rule in &config.rules {
        println!(
            "# scope: {} -> {} (weight: {}, style: {})",
            rule.scope,
            rule.font.family,
            weight_label(rule.font.weight),
            style_label(rule.font.style),
        );
        println!("# symbol_map U+XXXX-U+YYYY {}", rule.font.family);
        println!();
    }
}

fn cmd_dump(config: &PolyfontConfig) {
    let rules = config.to_rules();

    println!("Loaded {} rules (including default catchall):", rules.len());
    for (i, rule) in rules.iter().enumerate() {
        println!(
            "  [{}] scope: {} -> {} (weight: {}, style: {})",
            i,
            rule.scope,
            rule.font.family,
            weight_label(rule.font.weight),
            style_label(rule.font.style),
        );
    }

    #[allow(clippy::similar_names)]
    let scope_resolver = ScopeResolver::from_rules(rules);

    let test_scopes = [
        "keyword",
        "keyword.control",
        "keyword.operator",
        "comment",
        "comment.line",
        "comment.block",
        "string",
        "string.quoted",
        "string.quoted.double",
        "entity.name.function",
        "entity.name.type",
        "variable",
        "variable.other",
        "variable.parameter",
        "constant",
        "constant.numeric",
        "punctuation",
        "punctuation.separator",
        "entity.name.tag",
        "entity.other.attribute-name",
        "support.function",
    ];

    println!("\nScope resolution:");
    for scope in &test_scopes {
        match scope_resolver.resolve(scope) {
            Some(matched) => {
                println!(
                    "  {} -> {} (specificity: {}, rule: {})",
                    scope,
                    matched.assignment.font.family,
                    matched.assignment.specificity,
                    matched.rule_index,
                );
            }
            None => {
                println!("  {scope} -> (no match)");
            }
        }
    }
}

fn cmd_font_list() -> Result<()> {
    let discovery = create_discovery();
    let families = discovery.list_families()?;
    if families.is_empty() {
        println!("No font families found.");
    } else {
        for family in &families {
            println!("{family}");
        }
    }
    Ok(())
}

fn cmd_font_check(families: &[String]) -> Result<()> {
    let scanner = FontScanner::new();
    for family in families {
        if scanner.is_available(family) {
            println!("[ok] {family}");
        } else {
            println!("[missing] {family}");
        }
    }
    Ok(())
}

fn cmd_theme_list() {
    let registry = ThemeRegistry;
    let themes = registry.list_themes();
    for theme in &themes {
        println!("{} - {}", theme.name, theme.description);
    }
}

fn cmd_theme_show(name: &str) -> Result<()> {
    let registry = ThemeRegistry;
    let theme = registry
        .get_theme(name)
        .with_context(|| format!("theme '{name}' not found"))?;
    let config = ThemeExporter::export_config(&theme.config)?;
    println!("{config}");
    Ok(())
}

fn cmd_theme_apply(name: &str) -> Result<()> {
    let registry = ThemeRegistry;
    let theme = registry
        .get_theme(name)
        .with_context(|| format!("theme '{name}' not found"))?;
    let config = ThemeExporter::export_config(&theme.config)?;
    let dest = std::env::current_dir()?.join(".polyfont.toml");
    std::fs::write(&dest, config).with_context(|| format!("failed to write {}", dest.display()))?;
    eprintln!("Theme '{name}' written to {}", dest.display());
    Ok(())
}

fn cmd_font_install(families: &[String]) -> Result<()> {
    use polyfont_fonts::download::FontDownloader;
    let downloader = FontDownloader::new();
    let known = FontDownloader::list_available_sources();

    for family in families {
        let source_info = known.iter().find(|s| {
            s.name.eq_ignore_ascii_case(family)
                || s.name
                    .replace(' ', "")
                    .eq_ignore_ascii_case(&family.replace(' ', ""))
        });

        let Some(info) = source_info else {
            eprintln!(
                "[skip] {family} -- not in known sources. Known: {}",
                known
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            continue;
        };

        println!("Downloading {} (license: {})...", info.name, info.license);
        match downloader.download_to_cache(&info.name, info.source.clone()) {
            Ok(path) => println!("  [ok] cached at {}", path.display()),
            Err(e) => eprintln!("  [error] {e}"),
        }
    }
    Ok(())
}

fn cmd_font_preview(family: &str, text: &str) -> Result<()> {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let width = text.len() as u32 * 9;
    let height = 30u32;

    let svg = [
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        &format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">"
        ),
        "<rect width=\"100%\" height=\"100%\" fill=\"#1e1e2e\"/>",
        &format!("<text x=\"4\" y=\"20\" font-family=\"{family}, monospace\" font-size=\"14\" fill=\"#cdd6f4\">{escaped}</text>"),
        "</svg>",
    ]
    .join("\n");

    println!("{svg}");
    Ok(())
}

fn cmd_theme_import(path: &PathBuf, mapping_path: Option<&PathBuf>) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let font_mapping = if let Some(mp) = mapping_path {
        let mapping_content = std::fs::read_to_string(mp)
            .with_context(|| format!("failed to read mapping {}", mp.display()))?;
        toml::from_str::<polyfont_themes::FontMapping>(&mapping_content)
            .context("failed to parse font mapping TOML")?
    } else {
        polyfont_themes::FontMapping::default()
    };

    let importer = polyfont_themes::ThemeImporter;
    let config = if path.extension().is_some_and(|e| e == "json") {
        importer
            .import_vscode_theme(&content, &font_mapping)
            .context("failed to import VSCode theme")?
    } else {
        importer
            .import_textmate_theme(&content, &font_mapping)
            .context("failed to import TextMate theme")?
    };

    let output = ThemeExporter::export_config(&config)?;
    let dest = std::env::current_dir()?.join(".polyfont.toml");
    std::fs::write(&dest, &output).with_context(|| "failed to write .polyfont.toml")?;
    eprintln!(
        "Imported {} rules from {} -> .polyfont.toml",
        config.rules.len(),
        path.display()
    );
    Ok(())
}

fn cmd_theme_export(config: &PolyfontConfig, format: &str, output: Option<&PathBuf>) -> Result<()> {
    let content = match format {
        "vscode" | "json" => ThemeExporter::export_vscode(config)?,
        _ => ThemeExporter::export_config(config)?,
    };

    match output {
        Some(path) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create directory {}", parent.display()))?;
            }
            std::fs::write(path, &content)
                .with_context(|| format!("failed to write {}", path.display()))?;
            eprintln!("Exported to {}", path.display());
        }
        None => println!("{content}"),
    }
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Check => {
            let config = load_config(cli.config.as_ref())?;
            cmd_check(&config);
            Ok(())
        }
        Commands::Vscode { output } => {
            let config = load_config(cli.config.as_ref())?;
            cmd_vscode(&config, output.as_ref())
        }
        Commands::Neovim => {
            let config = load_config(cli.config.as_ref())?;
            cmd_neovim(&config);
            Ok(())
        }
        Commands::Kitty => {
            let config = load_config(cli.config.as_ref())?;
            cmd_kitty(&config);
            Ok(())
        }
        Commands::Dump => {
            let config = load_config(cli.config.as_ref())?;
            cmd_dump(&config);
            Ok(())
        }
        Commands::Font { action } => match action {
            FontAction::List => cmd_font_list(),
            FontAction::Check { family } => cmd_font_check(&family),
            FontAction::Install { family } => cmd_font_install(&family),
            FontAction::Preview { family, text } => cmd_font_preview(&family, &text),
        },
        Commands::Theme { action } => match action {
            ThemeAction::List => {
                cmd_theme_list();
                Ok(())
            }
            ThemeAction::Show { name } => cmd_theme_show(&name),
            ThemeAction::Apply { name } => cmd_theme_apply(&name),
            ThemeAction::Import { path, mapping } => cmd_theme_import(&path, mapping.as_ref()),
            ThemeAction::Export { format, output } => {
                let config = load_config(cli.config.as_ref())?;
                cmd_theme_export(&config, &format, output.as_ref())
            }
        },
    }
}
