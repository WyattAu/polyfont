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
        Err(format!(
            "polyfont: command '{}' is not supported. Per-scope font overrides are not yet available in Zed's extension API. Use 'polyfont vscode' to generate JSON and adapt it into a Zed theme manually.",
            command.command
        ).into())
    }
}

zed::register_extension!(PolyfontExtension);
