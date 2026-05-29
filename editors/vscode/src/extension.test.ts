import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { parsePolyfontConfig, FontWeight } from "./configParser";

type PolyfontConfig = import("./configParser").PolyfontConfig;

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

describe("isBoldWeight", () => {
  it("returns false for undefined", () => {
    assert.equal(isBoldWeight(undefined), false);
  });

  it("returns true for bold", () => {
    assert.equal(isBoldWeight("bold"), true);
  });

  it("returns true for semi-bold", () => {
    assert.equal(isBoldWeight("semi-bold"), true);
  });

  it("returns true for extra-bold", () => {
    assert.equal(isBoldWeight("extra-bold"), true);
  });

  it("returns true for black", () => {
    assert.equal(isBoldWeight("black"), true);
  });

  it("returns false for regular", () => {
    assert.equal(isBoldWeight("regular"), false);
  });

  it("returns false for medium", () => {
    assert.equal(isBoldWeight("medium"), false);
  });

  it("returns false for light", () => {
    assert.equal(isBoldWeight("light"), false);
  });

  it("returns false for thin", () => {
    assert.equal(isBoldWeight("thin"), false);
  });

  it("returns false for extra-light", () => {
    assert.equal(isBoldWeight("extra-light"), false);
  });
});

describe("buildTextMateRules", () => {
  const minimalConfig = (rules: string) =>
    parsePolyfontConfig(`version = 1\n${rules}`);

  it("produces empty array for no rules", () => {
    const config = parsePolyfontConfig("version = 1");
    const rules = buildTextMateRules(config);
    assert.equal(rules.length, 0);
  });

  it("builds a single rule with scope and fontFamily", () => {
    const config = minimalConfig(`
[[rules]]
scope = "keyword"
[rules.font]
family = "Fira Code"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules.length, 1);
    assert.equal(rules[0].scope, "keyword");
    assert.equal(rules[0].settings.fontFamily, "Fira Code");
  });

  it("includes fallbacks in fontFamily", () => {
    const config = minimalConfig(`
[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
fallbacks = ["JetBrains Mono", "monospace"]
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontFamily, "Maple Mono, JetBrains Mono, monospace");
  });

  it("adds bold fontStyle for bold weight", () => {
    const config = minimalConfig(`
[[rules]]
scope = "keyword"
[rules.font]
family = "Fira Code"
weight = "bold"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontStyle, "bold");
    assert.equal(rules[0].settings.fontWeight, "bold");
  });

  it("adds bold fontStyle for semi-bold weight", () => {
    const config = minimalConfig(`
[[rules]]
scope = "keyword"
[rules.font]
family = "Fira Code"
weight = "semi-bold"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontStyle, "bold");
  });

  it("adds bold fontStyle for extra-bold weight", () => {
    const config = minimalConfig(`
[[rules]]
scope = "keyword"
[rules.font]
family = "Fira Code"
weight = "extra-bold"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontStyle, "bold");
  });

  it("adds bold fontStyle for black weight", () => {
    const config = minimalConfig(`
[[rules]]
scope = "keyword"
[rules.font]
family = "Fira Code"
weight = "black"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontStyle, "bold");
  });

  it("adds italic fontStyle for italic style", () => {
    const config = minimalConfig(`
[[rules]]
scope = "comment"
[rules.font]
family = "IBM Plex Mono"
style = "italic"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontStyle, "italic");
  });

  it("adds italic fontStyle for oblique style", () => {
    const config = minimalConfig(`
[[rules]]
scope = "comment"
[rules.font]
family = "IBM Plex Mono"
style = "oblique"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontStyle, "italic");
  });

  it("combines bold and italic in fontStyle", () => {
    const config = minimalConfig(`
[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
weight = "bold"
style = "italic"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontStyle, "bold italic");
  });

  it("omits fontStyle for regular weight and normal style", () => {
    const config = minimalConfig(`
[[rules]]
scope = "string"
[rules.font]
family = "Source Code Pro"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontStyle, undefined);
  });

  it("omits fontWeight for regular weight", () => {
    const config = minimalConfig(`
[[rules]]
scope = "string"
[rules.font]
family = "Source Code Pro"
weight = "regular"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontWeight, undefined);
  });

  it("sets fontWeight for non-regular non-bold weight", () => {
    const config = minimalConfig(`
[[rules]]
scope = "variable"
[rules.font]
family = "Monaspace Neon"
weight = "light"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].settings.fontWeight, "light");
    assert.equal(rules[0].settings.fontStyle, undefined);
  });

  it("builds multiple rules in order", () => {
    const config = minimalConfig(`
[[rules]]
scope = "keyword"
[rules.font]
family = "A"
[[rules]]
scope = "comment"
[rules.font]
family = "B"
[[rules]]
scope = "string"
[rules.font]
family = "C"
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules.length, 3);
    assert.equal(rules[0].scope, "keyword");
    assert.equal(rules[1].scope, "comment");
    assert.equal(rules[2].scope, "string");
  });

  it("handles rule with all fields populated", () => {
    const config = minimalConfig(`
[[rules]]
scope = "entity.name.function"
[rules.font]
family = "JetBrains Mono"
weight = "semi-bold"
style = "italic"
fallbacks = ["Fira Code", "monospace"]
`);
    const rules = buildTextMateRules(config);
    assert.equal(rules[0].scope, "entity.name.function");
    assert.equal(rules[0].settings.fontFamily, "JetBrains Mono, Fira Code, monospace");
    assert.equal(rules[0].settings.fontStyle, "bold italic");
    assert.equal(rules[0].settings.fontWeight, "semi-bold");
  });
});
