use zed_extension_api::{self as zed, Result};

struct PolyfontExtension;

impl zed::Extension for PolyfontExtension {
    fn new() -> Self {
        Self
    }

    fn language_command(
        &mut self,
        command: zed::LanguageCommand,
        _worktree: &zed::Worktree,
    ) -> Result<serde_json::Value> {
        match command {
            zed::LanguageCommand::Apply => {
                Err("polyfont: per-scope font overrides are not yet supported by Zed's extension API. Use 'polyfont vscode' to generate JSON and adapt it into a Zed theme manually.".into())
            }
        }
    }
}

zed::register_extension!(PolyfontExtension);
