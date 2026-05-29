package com.polyfont.intellij

object PolyfontConfigParser {

    data class Rule(
        val scope: String,
        val fontFamily: String,
        val weight: String? = null,
        val style: String? = null,
    )

    fun parse(toml: String): List<Rule> {
        val rules = mutableListOf<Rule>()
        val lines = toml.lines()
        var inRulesArray = false
        var inRulesFont = false
        var currentScope: String? = null
        var currentFont: String? = null
        var currentWeight: String? = null
        var currentStyle: String? = null

        for (line in lines) {
            val trimmed = line.trim()

            if (trimmed == "[[rules]]") {
                if (inRulesArray && currentScope != null && currentFont != null) {
                    rules.add(Rule(currentScope, currentFont, currentWeight, currentStyle))
                }
                inRulesArray = true
                inRulesFont = false
                currentScope = null
                currentFont = null
                currentWeight = null
                currentStyle = null
                continue
            }

            if (trimmed == "[rules.font]") {
                inRulesFont = true
                continue
            }

            if (trimmed.startsWith("[[") || (trimmed.startsWith("[") && !trimmed.startsWith("[["))) {
                if (inRulesArray && currentScope != null && currentFont != null) {
                    rules.add(Rule(currentScope, currentFont, currentWeight, currentStyle))
                }
                inRulesArray = false
                inRulesFont = false
                currentScope = null
                currentFont = null
                currentWeight = null
                currentStyle = null
                continue
            }

            if (inRulesArray) {
                val scopeMatch = Regex("""^scope\s*=\s*"([^"]+)""""").find(trimmed)
                if (scopeMatch != null) {
                    currentScope = scopeMatch.groupValues[1]
                }

                if (inRulesFont) {
                    val familyMatch = Regex("""^family\s*=\s*"([^"]+)""""").find(trimmed)
                    if (familyMatch != null) {
                        currentFont = familyMatch.groupValues[1]
                    }
                    val weightMatch = Regex("""^weight\s*=\s*"([^"]+)""""").find(trimmed)
                    if (weightMatch != null) {
                        currentWeight = weightMatch.groupValues[1]
                    }
                    val styleMatch = Regex("""^style\s*=\s*"([^"]+)""""").find(trimmed)
                    if (styleMatch != null) {
                        currentStyle = styleMatch.groupValues[1]
                    }
                } else {
                    val fontFamilyMatch = Regex("""^font\.family\s*=\s*"([^"]+)""""").find(trimmed)
                    if (fontFamilyMatch != null) {
                        currentFont = fontFamilyMatch.groupValues[1]
                    }
                }
            }
        }

        if (inRulesArray && currentScope != null && currentFont != null) {
            rules.add(Rule(currentScope, currentFont, currentWeight, currentStyle))
        }

        return rules
    }
}
