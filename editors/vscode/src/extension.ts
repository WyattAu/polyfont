import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import { parsePolyfontConfig, validateConfig, PolyfontConfig, FontWeight } from "./configParser";

const CONFIG_FILENAME = ".polyfont.toml";

let watcher: vscode.FileSystemWatcher | undefined;
let statusBarItem: vscode.StatusBarItem | undefined;
let debounceTimer: ReturnType<typeof setTimeout> | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const applyCmd = vscode.commands.registerCommand("polyfont.applyFonts", () =>
    applyFonts()
  );
  const generateCmd = vscode.commands.registerCommand(
    "polyfont.generateConfig",
    () => generateConfig()
  );

  context.subscriptions.push(applyCmd, generateCmd);

  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100
  );
  statusBarItem.text = "$(symbol-font) Polyfont";
  statusBarItem.tooltip = "Polyfont - Click to reapply font rules";
  statusBarItem.command = "polyfont.applyFonts";
  context.subscriptions.push(statusBarItem);
  statusBarItem.show();

  setupWatcher(context);

  applyFonts();
}

export function deactivate(): void {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
}

function findConfigFile(): string | undefined {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (workspaceFolders) {
    for (const folder of workspaceFolders) {
      let dir = folder.uri.fsPath;
      while (true) {
        const candidate = path.join(dir, CONFIG_FILENAME);
        if (fs.existsSync(candidate)) {
          return candidate;
        }
        const parent = path.dirname(dir);
        if (parent === dir) break;
        dir = parent;
      }
    }
  }

  const homeConfig = path.join(os.homedir(), CONFIG_FILENAME);
  if (fs.existsSync(homeConfig)) {
    return homeConfig;
  }

  return undefined;
}

function isBoldWeight(weight: FontWeight | undefined): boolean {
  if (!weight) return false;
  return ["bold", "semi-bold", "extra-bold", "black"].includes(weight);
}

function buildTextMateRules(config: PolyfontConfig): object[] {
  const rules: object[] = [];

  for (const rule of config.rules) {
    const families = [rule.font.family, ...(rule.font.fallbacks || [])];
    const fontFamily = families.join(", ");

    const settings: Record<string, string> = {
      fontFamily,
    };

    const fontStyleParts: string[] = [];
    if (isBoldWeight(rule.font.weight)) {
      fontStyleParts.push("bold");
    }
    if (rule.font.style === "italic" || rule.font.style === "oblique") {
      fontStyleParts.push("italic");
    }
    if (fontStyleParts.length > 0) {
      settings.fontStyle = fontStyleParts.join(" ");
    }

    if (rule.font.weight && rule.font.weight !== "regular") {
      settings.fontWeight = rule.font.weight;
    }

    rules.push({
      scope: rule.scope,
      settings,
    });
  }

  return rules;
}

async function applyFonts(): Promise<void> {
  const configPath = findConfigFile();
  if (!configPath) {
    if (statusBarItem) {
      statusBarItem.text = "$(symbol-font) Polyfont: No config";
    }
    return;
  }

  try {
    const content = fs.readFileSync(configPath, "utf-8");
    const config = parsePolyfontConfig(content);

    const errors = validateConfig(config);
    if (errors.length > 0) {
      vscode.window.showErrorMessage(
        `Polyfont config errors:\n${errors.join("\n")}`
      );
      return;
    }

    const textMateRules = buildTextMateRules(config);

    const cfg = vscode.workspace.getConfiguration();
    const existing =
      cfg.get<Record<string, any>>("editor.tokenColorCustomizations") || {};

    const merged = {
      ...existing,
      textMateRules,
    };

    const target = vscode.workspace.workspaceFolders
      ? vscode.ConfigurationTarget.Workspace
      : vscode.ConfigurationTarget.Global;

    await cfg.update("editor.tokenColorCustomizations", merged, target);

    if (statusBarItem) {
      statusBarItem.text = `$(symbol-font) Polyfont (${textMateRules.length} rules)`;
      statusBarItem.tooltip = `Polyfont: ${textMateRules.length} font rules active\nConfig: ${configPath}\nClick to reapply`;
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    vscode.window.showErrorMessage(`Polyfont error: ${message}`);
    if (statusBarItem) {
      statusBarItem.text = "$(error) Polyfont: Error";
    }
  }
}

function setupWatcher(context: vscode.ExtensionContext): void {
  watcher = vscode.workspace.createFileSystemWatcher(
    `**/${CONFIG_FILENAME}`,
    false,
    false,
    false
  );

  const debouncedApply = () => {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => applyFonts(), 500);
  };

  watcher.onDidCreate(debouncedApply);
  watcher.onDidChange(debouncedApply);
  watcher.onDidDelete(() => {
    if (statusBarItem) {
      statusBarItem.text = "$(symbol-font) Polyfont: No config";
    }
  });

  context.subscriptions.push(watcher);
}

const SAMPLE_CONFIG = `# Polyfont Configuration
# See: https://github.com/WyattAu/polyfont
#
# Font weights: thin, extra-light, light, regular, medium,
#               semi-bold, bold, extra-bold, black
# Font styles:  normal, italic, oblique

version = 1

[default]
family = "Fira Code"
fallbacks = ["JetBrains Mono", "monospace"]

[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
weight = "bold"

[[rules]]
scope = "comment"
[rules.font]
family = "IBM Plex Mono"
style = "italic"

[[rules]]
scope = "string"
[rules.font]
family = "Source Code Pro"

[[rules]]
scope = "entity.name.function"
[rules.font]
family = "JetBrains Mono"
weight = "semi-bold"

[[rules]]
scope = "variable"
[rules.font]
family = "Monaspace Neon"

[[rules]]
scope = "constant"
[rules.font]
family = "Monaspace Argon"
weight = "bold"
`;

async function generateConfig(): Promise<void> {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) {
    vscode.window.showErrorMessage("Polyfont: Open a workspace first");
    return;
  }

  const targetDir = workspaceFolders[0].uri.fsPath;
  const targetPath = path.join(targetDir, CONFIG_FILENAME);

  if (fs.existsSync(targetPath)) {
    const choice = await vscode.window.showWarningMessage(
      `polyfont: ${CONFIG_FILENAME} already exists. Overwrite?`,
      "Overwrite",
      "Cancel"
    );
    if (choice !== "Overwrite") return;
  }

  fs.writeFileSync(targetPath, SAMPLE_CONFIG, "utf-8");

  const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(targetPath));
  await vscode.window.showTextDocument(doc);

  vscode.window.showInformationMessage(
    `Polyfont: Generated ${CONFIG_FILENAME}. Edit and save to apply.`
  );

  applyFonts();
}
