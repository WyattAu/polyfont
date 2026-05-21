export type FontWeight =
  | "thin"
  | "extra-light"
  | "light"
  | "regular"
  | "medium"
  | "semi-bold"
  | "bold"
  | "extra-bold"
  | "black";

export type FontStyle = "normal" | "italic" | "oblique";

export interface FontConfig {
  family: string;
  fallbacks?: string[];
  weight?: FontWeight;
  style?: FontStyle;
  size?: number;
}

export interface DefaultFontConfig {
  family: string;
  fallbacks?: string[];
  weight?: FontWeight;
  style?: FontStyle;
  size?: number;
}

export interface RuleConfig {
  scope: string;
  font: FontConfig;
}

export interface PolyfontConfig {
  version: number;
  default?: DefaultFontConfig;
  rules: RuleConfig[];
}

type TomlValue = string | number | boolean | TomlValue[] | { [key: string]: TomlValue };

function parseTomlValue(raw: string): TomlValue {
  raw = raw.trim();

  if (raw.startsWith('"') && raw.endsWith('"')) {
    return unescapeString(raw.slice(1, -1));
  }
  if (raw.startsWith("'") && raw.endsWith("'")) {
    return raw.slice(1, -1);
  }

  if (raw.startsWith("[")) {
    return parseInlineArray(raw);
  }

  if (raw === "true") return true;
  if (raw === "false") return false;

  const num = Number(raw);
  if (!isNaN(num) && raw !== "") return num;

  return raw;
}

function unescapeString(s: string): string {
  return s
    .replace(/\\n/g, "\n")
    .replace(/\\t/g, "\t")
    .replace(/\\r/g, "\r")
    .replace(/\\\\/g, "\\")
    .replace(/\\"/g, '"');
}

function parseInlineArray(raw: string): TomlValue[] {
  const inner = raw.slice(1, raw.length - 1).trim();
  if (!inner) return [];

  const result: TomlValue[] = [];
  let current = "";
  let inString = false;
  let stringChar = "";
  let depth = 0;

  for (let i = 0; i < inner.length; i++) {
    const ch = inner[i];

    if (inString) {
      current += ch;
      if (ch === stringChar && inner[i - 1] !== "\\") {
        inString = false;
      }
      continue;
    }

    if (ch === '"' || ch === "'") {
      inString = true;
      stringChar = ch;
      current += ch;
    } else if (ch === "[") {
      depth++;
      current += ch;
    } else if (ch === "]") {
      depth--;
      current += ch;
    } else if (ch === "," && depth === 0) {
      const trimmed = current.trim();
      if (trimmed) result.push(parseTomlValue(trimmed));
      current = "";
    } else {
      current += ch;
    }
  }

  const trimmed = current.trim();
  if (trimmed) result.push(parseTomlValue(trimmed));

  return result;
}

function navigateOrCreate(obj: Record<string, any>, path: string[]): Record<string, any> {
  let current: any = obj;
  for (const key of path) {
    if (current[key] === undefined) {
      current[key] = {};
    }
    if (Array.isArray(current[key])) {
      const arr = current[key] as any[];
      current = arr[arr.length - 1];
    } else if (typeof current[key] === "object" && current[key] !== null) {
      current = current[key];
    }
  }
  return current as Record<string, any>;
}

function setValue(obj: Record<string, any>, key: string, value: TomlValue): void {
  if (key.includes(".")) {
    const parts = key.split(".");
    let current: any = obj;
    for (let i = 0; i < parts.length - 1; i++) {
      if (current[parts[i]] === undefined || typeof current[parts[i]] !== "object") {
        current[parts[i]] = {};
      }
      current = current[parts[i]];
    }
    current[parts[parts.length - 1]] = value;
  } else {
    obj[key] = value;
  }
}

export function parsePolyfontConfig(input: string): PolyfontConfig {
  const root: Record<string, any> = {};
  let current: Record<string, any> = root;

  const lines = input.split("\n");

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;

    const arrayTableMatch = trimmed.match(/^\[\[(.+)\]\]$/);
    if (arrayTableMatch) {
      const path = arrayTableMatch[1].split(".").map((s) => s.trim());
      const lastKey = path[path.length - 1];
      const parentPath = path.slice(0, -1);
      const parent =
        parentPath.length === 0 ? root : navigateOrCreate(root, parentPath);
      if (!parent[lastKey]) {
        parent[lastKey] = [];
      }
      const newElement: Record<string, any> = {};
      parent[lastKey].push(newElement);
      current = newElement;
      continue;
    }

    const tableMatch = trimmed.match(/^\[([^\[\]]+)\]$/);
    if (tableMatch) {
      const path = tableMatch[1].split(".").map((s) => s.trim());
      current = navigateOrCreate(root, path);
      continue;
    }

    const kvMatch = trimmed.match(/^([^\s=]+)\s*=\s*(.+)$/);
    if (kvMatch) {
      const key = kvMatch[1].trim();
      const rawValue = kvMatch[2].trim();
      const value = parseTomlValue(rawValue);
      setValue(current, key, value);
    }
  }

  if (root.version !== 1) {
    throw new Error(
      `Unsupported polyfont config version: ${root.version ?? "missing"}. Expected version = 1`
    );
  }

  if (!Array.isArray(root.rules)) {
    root.rules = [];
  }

  return root as unknown as PolyfontConfig;
}

const VALID_WEIGHTS: FontWeight[] = [
  "thin",
  "extra-light",
  "light",
  "regular",
  "medium",
  "semi-bold",
  "bold",
  "extra-bold",
  "black",
];

export function validateConfig(config: PolyfontConfig): string[] {
  const errors: string[] = [];

  if (config.default) {
    if (!config.default.family?.trim()) {
      errors.push("default font family must not be empty");
    }
    if (config.default.weight && !VALID_WEIGHTS.includes(config.default.weight)) {
      errors.push(`default font has invalid weight: "${config.default.weight}"`);
    }
    if (
      config.default.style &&
      !["normal", "italic", "oblique"].includes(config.default.style)
    ) {
      errors.push(`default font has invalid style: "${config.default.style}"`);
    }
  }

  for (let i = 0; i < config.rules.length; i++) {
    const rule = config.rules[i];
    if (!rule.scope?.trim()) {
      errors.push(`rule at index ${i} has an empty scope`);
    }
    if (!rule.font?.family?.trim()) {
      errors.push(
        `rule at index ${i} (scope "${rule.scope}") has an empty font family`
      );
    }
    if (rule.font?.weight && !VALID_WEIGHTS.includes(rule.font.weight)) {
      errors.push(
        `rule at index ${i} (scope "${rule.scope}") has invalid weight: "${rule.font.weight}"`
      );
    }
    if (
      rule.font?.style &&
      !["normal", "italic", "oblique"].includes(rule.font.style)
    ) {
      errors.push(
        `rule at index ${i} (scope "${rule.scope}") has invalid style: "${rule.font.style}"`
      );
    }
  }

  return errors;
}
