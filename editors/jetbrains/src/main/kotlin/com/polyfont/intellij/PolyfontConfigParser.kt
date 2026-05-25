package com.polyfont.intellij

object PolyfontConfigParser {

    data class Rule(val scope: String, val fontFamily: String)

    fun parse(toml: String): Map<String, String> {
        val rules = mutableListOf<Rule>()
        val lines = toml.lines()
        var inRulesArray = false
        var currentScope: String? = null
        var currentFont: String? = null

        for (line in lines) {
            val trimmed = line.trim()

            if (trimmed == "[[rules]]") {
                if (inRulesArray && currentScope != null && currentFont != null) {
                    rules.add(Rule(currentScope, currentFont))
                }
                inRulesArray = true
                currentScope = null
                currentFont = null
                continue
            }

            if (trimmed.startsWith("[[") || (trimmed.startsWith("[") && !trimmed.startsWith("[["))) {
                if (inRulesArray && currentScope != null && currentFont != null) {
                    rules.add(Rule(currentScope, currentFont))
                }
                inRulesArray = false
                currentScope = null
                currentFont = null
                continue
            }

            if (inRulesArray) {
                val scopeMatch = Regex("""^scope\s*=\s*"([^"]+)""""").find(trimmed)
                if (scopeMatch != null) {
                    currentScope = scopeMatch.groupValues[1]
                }

                val fontFamilyMatch = Regex("""^font\.family\s*=\s*"([^"]+)""""").find(trimmed)
                if (fontFamilyMatch != null) {
                    currentFont = fontFamilyMatch.groupValues[1]
                }
            }
        }

        if (inRulesArray && currentScope != null && currentFont != null) {
            rules.add(Rule(currentScope, currentFont))
        }

        return rules.associate { it.scope to it.fontFamily }
    }
}
