local toml = require("polyfont.toml")
local treesitter = require("polyfont.treesitter")

describe("polyfont", function()
    describe("toml parser", function()
        it("parses simple key-value strings", function()
            local result = toml.parse('key = "value"')
            assert.equal(result.key, "value")
        end)

        it("parses single-quoted strings", function()
            local result = toml.parse("key = 'value'")
            assert.equal(result.key, "value")
        end)

        it("parses integer values", function()
            local result = toml.parse("version = 42")
            assert.equal(result.version, 42)
        end)

        it("parses float values", function()
            local result = toml.parse("ratio = 3.14")
            assert.equal(result.ratio, 3.14)
        end)

        it("parses boolean true", function()
            local result = toml.parse("enabled = true")
            assert.equal(result.enabled, true)
        end)

        it("parses boolean false", function()
            local result = toml.parse("enabled = false")
            assert.equal(result.enabled, false)
        end)

        it("parses nested tables", function()
            local input = [[
[default]
family = "Fira Code"
weight = "bold"
]]
            local result = toml.parse(input)
            assert.equal(result.default.family, "Fira Code")
            assert.equal(result.default.weight, "bold")
        end)

        it("parses array of tables [[rules]]", function()
            local input = [[
[[rules]]
scope = "keyword"
[[rules]]
scope = "comment"
]]
            local result = toml.parse(input)
            assert.truthy(result.rules)
            assert.equal(#result.rules, 2)
            assert.equal(result.rules[1].scope, "keyword")
            assert.equal(result.rules[2].scope, "comment")
        end)

        it("parses inline arrays", function()
            local result = toml.parse('fallbacks = ["JetBrains Mono", "monospace"]')
            assert.truthy(result.fallbacks)
            assert.equal(#result.fallbacks, 2)
            assert.equal(result.fallbacks[1], "JetBrains Mono")
            assert.equal(result.fallbacks[2], "monospace")
        end)

        it("strips comments", function()
            local result = toml.parse('family = "Fira Code" # this is a comment')
            assert.equal(result.family, "Fira Code")
        end)

        it("ignores blank lines", function()
            local result = toml.parse("\n\nversion = 1\n\n")
            assert.equal(result.version, 1)
        end)

        it("parses nested array-of-tables with sub-tables", function()
            local input = [[
[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
weight = "bold"
]]
            local result = toml.parse(input)
            assert.equal(result.rules[1].scope, "keyword")
            assert.equal(result.rules[1].font.family, "Maple Mono")
            assert.equal(result.rules[1].font.weight, "bold")
        end)

        it("parses negative numbers", function()
            local result = toml.parse("offset = -5")
            assert.equal(result.offset, -5)
        end)

        it("parses escaped double quotes in strings", function()
            local result = toml.parse('name = "hello \\"world\\""')
            assert.equal(result.name, 'hello "world"')
        end)

        it("returns empty table for empty input", function()
            local result = toml.parse("")
            assert.truthy(type(result) == "table")
        end)

        it("errors on invalid array-of-tables syntax", function()
            assert.has_error(function()
                toml.parse("[[rules")
            end)
        end)
    end)

    describe("treesitter", function()
        describe("is_bold", function()
            it("returns true for bold", function()
                assert.equal(treesitter.is_bold("bold"), true)
            end)

            it("returns true for semi-bold", function()
                assert.equal(treesitter.is_bold("semi-bold"), true)
            end)

            it("returns true for extra-bold", function()
                assert.equal(treesitter.is_bold("extra-bold"), true)
            end)

            it("returns true for black", function()
                assert.equal(treesitter.is_bold("black"), true)
            end)

            it("returns true for medium", function()
                assert.equal(treesitter.is_bold("medium"), true)
            end)

            it("returns false for regular", function()
                assert.equal(treesitter.is_bold("regular"), false)
            end)

            it("returns false for light", function()
                assert.equal(treesitter.is_bold("light"), false)
            end)

            it("returns false for nil", function()
                assert.equal(treesitter.is_bold(nil), false)
            end)
        end)

        describe("is_italic", function()
            it("returns true for italic", function()
                assert.equal(treesitter.is_italic("italic"), true)
            end)

            it("returns true for oblique", function()
                assert.equal(treesitter.is_italic("oblique"), true)
            end)

            it("returns false for normal", function()
                assert.equal(treesitter.is_italic("normal"), false)
            end)

            it("returns false for nil", function()
                assert.equal(treesitter.is_italic(nil), false)
            end)
        end)

        describe("scope_matches", function()
            it("matches wildcard *", function()
                assert.equal(treesitter.scope_matches("keyword", "*"), true)
            end)

            it("matches exact scope", function()
                assert.equal(treesitter.scope_matches("keyword", "keyword"), true)
            end)

            it("rejects non-matching exact scope", function()
                assert.equal(treesitter.scope_matches("comment", "keyword"), false)
            end)

            it("matches hierarchical scope", function()
                assert.equal(treesitter.scope_matches("keyword.control.conditional", "keyword"), true)
            end)

            it("matches middle hierarchy", function()
                assert.equal(treesitter.scope_matches("keyword.control.conditional", "keyword.control"), true)
            end)

            it("rejects if scope has fewer parts than pattern", function()
                assert.equal(treesitter.scope_matches("keyword", "keyword.control"), false)
            end)

            it("matches negated pattern that does not match", function()
                assert.equal(treesitter.scope_matches("comment", "-keyword"), true)
            end)

            it("rejects negated pattern that matches", function()
                assert.equal(treesitter.scope_matches("keyword", "-keyword"), false)
            end)

            it("handles complex negated pattern", function()
                assert.equal(treesitter.scope_matches("keyword.control", "-keyword"), false)
            end)
        end)

        describe("scope_specificity", function()
            it("returns 1 for single-part scope", function()
                assert.equal(treesitter.scope_specificity("keyword"), 1)
            end)

            it("returns 3 for three-part scope", function()
                assert.equal(treesitter.scope_specificity("entity.name.function"), 3)
            end)

            it("returns correct count for deep scope", function()
                assert.equal(treesitter.scope_specificity("keyword.control.conditional"), 3)
            end)
        end)

        describe("scope_to_hl_group", function()
            it("converts simple scope to hl group", function()
                assert.equal(treesitter.scope_to_hl_group("keyword"), "Keyword")
            end)

            it("converts dotted scope to underscores with CamelCase", function()
                assert.equal(treesitter.scope_to_hl_group("entity.name.function"), "Entity_name_function")
            end)

            it("converts hyphenated scope to underscores", function()
                assert.equal(treesitter.scope_to_hl_group("keyword.function"), "Keyword_function")
            end)
        end)

        describe("scope_from_capture", function()
            it("maps keyword capture to keyword scope", function()
                assert.equal(treesitter.scope_from_capture("keyword"), "keyword")
            end)

            it("maps function capture to entity.name.function", function()
                assert.equal(treesitter.scope_from_capture("function"), "entity.name.function")
            end)

            it("returns nil for unknown capture", function()
                assert.equal(treesitter.scope_from_capture("nonexistent"), nil)
            end)
        end)

        describe("scope_matches_selector", function()
            it("matches first pattern in comma-separated selector", function()
                assert.equal(treesitter.scope_matches_selector("keyword", "keyword, comment"), true)
            end)

            it("matches second pattern in comma-separated selector", function()
                assert.equal(treesitter.scope_matches_selector("comment", "keyword, comment"), true)
            end)

            it("returns false when no pattern matches", function()
                assert.equal(treesitter.scope_matches_selector("string", "keyword, comment"), false)
            end)
        end)

        describe("resolve_scope", function()
            it("returns nil for empty rules", function()
                assert.equal(treesitter.resolve_scope("keyword", {}), nil)
            end)

            it("returns matching rule", function()
                local rules = {
                    { scope = "keyword", font = { family = "A" } },
                    { scope = "comment", font = { family = "B" } },
                }
                local rule = treesitter.resolve_scope("keyword", rules)
                assert.equal(rule.scope, "keyword")
            end)

            it("prefers more specific match", function()
                local rules = {
                    { scope = "keyword", font = { family = "A" } },
                    { scope = "keyword.control", font = { family = "B" } },
                }
                local rule = treesitter.resolve_scope("keyword.control.conditional", rules)
                assert.equal(rule.scope, "keyword.control")
            end)

            it("prefers earlier rule on equal specificity", function()
                local rules = {
                    { scope = "keyword", font = { family = "A" } },
                    { scope = "keyword", font = { family = "B" } },
                }
                local rule = treesitter.resolve_scope("keyword", rules)
                assert.equal(rule.font.family, "A")
            end)
        end)
    end)
end)
