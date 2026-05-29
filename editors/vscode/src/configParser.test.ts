import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { parsePolyfontConfig, validateConfig } from "./configParser";

describe("parsePolyfontConfig", () => {
  it("parses minimal valid config", () => {
    const input = `
version = 1
[[rules]]
scope = "keyword"
[rules.font]
family = "Fira Code"
`;
    const config = parsePolyfontConfig(input);
    assert.equal(config.version, 1);
    assert.equal(config.rules.length, 1);
    assert.equal(config.rules[0].scope, "keyword");
    assert.equal(config.rules[0].font.family, "Fira Code");
  });

  it("parses config with default font", () => {
    const input = `
version = 1
[default]
family = "JetBrains Mono"
fallbacks = ["monospace"]

[[rules]]
scope = "comment"
[rules.font]
family = "IBM Plex Mono"
style = "italic"
`;
    const config = parsePolyfontConfig(input);
    assert.equal(config.default?.family, "JetBrains Mono");
    assert.deepEqual(config.default?.fallbacks, ["monospace"]);
    assert.equal(config.rules[0].font.style, "italic");
  });

  it("parses font weight and style", () => {
    const input = `
version = 1
[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
weight = "bold"
style = "italic"
`;
    const config = parsePolyfontConfig(input);
    assert.equal(config.rules[0].font.weight, "bold");
    assert.equal(config.rules[0].font.style, "italic");
  });

  it("parses multiple rules", () => {
    const input = `
version = 1
[[rules]]
scope = "keyword"
[rules.font]
family = "Mono A"
[[rules]]
scope = "comment"
[rules.font]
family = "Mono B"
[[rules]]
scope = "string"
[rules.font]
family = "Mono C"
`;
    const config = parsePolyfontConfig(input);
    assert.equal(config.rules.length, 3);
    assert.equal(config.rules[0].scope, "keyword");
    assert.equal(config.rules[1].scope, "comment");
    assert.equal(config.rules[2].scope, "string");
  });

  it("rejects wrong version", () => {
    const input = `version = 2\n[[rules]]\nscope = "x"\n[rules.font]\nfamily = "y"`;
    assert.throws(() => parsePolyfontConfig(input), /Unsupported polyfont config version/);
  });

  it("handles empty rules", () => {
    const input = `version = 1`;
    const config = parsePolyfontConfig(input);
    assert.equal(config.rules.length, 0);
  });

  it("handles inline arrays for fallbacks", () => {
    const input = `
version = 1
[default]
family = "Fira Code"
fallbacks = ["JetBrains Mono", "IBM Plex Mono", "monospace"]
`;
    const config = parsePolyfontConfig(input);
    assert.deepEqual(config.default?.fallbacks, [
      "JetBrains Mono",
      "IBM Plex Mono",
      "monospace",
    ]);
  });

  it("handles comments", () => {
    const input = `
# This is a comment
version = 1
# Another comment
[[rules]]
scope = "keyword"
[rules.font]
family = "Fira Code"
`;
    const config = parsePolyfontConfig(input);
    assert.equal(config.version, 1);
    assert.equal(config.rules.length, 1);
  });
});

describe("validateConfig", () => {
  it("returns no errors for valid config", () => {
    const input = `
version = 1
[default]
family = "Fira Code"
[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
`;
    const config = parsePolyfontConfig(input);
    const errors = validateConfig(config);
    assert.deepEqual(errors, []);
  });

  it("detects empty default family", () => {
    const input = `
version = 1
[default]
family = ""
`;
    const config = parsePolyfontConfig(input);
    const errors = validateConfig(config);
    assert.ok(errors.some((e) => e.includes("default font family")));
  });

  it("detects empty rule scope", () => {
    const input = `
version = 1
[[rules]]
scope = ""
[rules.font]
family = "Fira Code"
`;
    const config = parsePolyfontConfig(input);
    const errors = validateConfig(config);
    assert.ok(errors.some((e) => e.includes("empty scope")));
  });

  it("detects empty rule font family", () => {
    const input = `
version = 1
[[rules]]
scope = "keyword"
[rules.font]
family = ""
`;
    const config = parsePolyfontConfig(input);
    const errors = validateConfig(config);
    assert.ok(errors.some((e) => e.includes("empty font family")));
  });

  it("detects invalid weight", () => {
    const input = `
version = 1
[default]
family = "Fira Code"
weight = "ultra-bold"
`;
    const config = parsePolyfontConfig(input);
    const errors = validateConfig(config);
    assert.ok(errors.some((e) => e.includes("invalid weight")));
  });

  it("detects invalid style", () => {
    const input = `
version = 1
[default]
family = "Fira Code"
style = "underline"
`;
    const config = parsePolyfontConfig(input);
    const errors = validateConfig(config);
    assert.ok(errors.some((e) => e.includes("invalid style")));
  });

  it("validates all rule weights", () => {
    const input = `
version = 1
[[rules]]
scope = "keyword"
[rules.font]
family = "A"
weight = "super-bold"
[[rules]]
scope = "comment"
[rules.font]
family = "B"
weight = "medium"
`;
    const config = parsePolyfontConfig(input);
    const errors = validateConfig(config);
    assert.equal(errors.length, 1);
    assert.ok(errors[0].includes("super-bold"));
  });
});
