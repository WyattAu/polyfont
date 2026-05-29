package com.polyfont.intellij

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Assertions.*

class PolyfontConfigParserTest {

    @Test
    fun emptyInputReturnsEmptyList() {
        val result = PolyfontConfigParser.parse("")
        assertTrue(result.isEmpty())
    }

    @Test
    fun noRulesBlocksReturnsEmptyList() {
        val toml = "version = 1\n"
        val result = PolyfontConfigParser.parse(toml)
        assertTrue(result.isEmpty())
    }

    @Test
    fun singleRuleWithMinimalFields() {
        val toml = """
            [[rules]]
            scope = "keyword"
            [rules.font]
            family = "Fira Code"
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(1, result.size)
        assertEquals("keyword", result[0].scope)
        assertEquals("Fira Code", result[0].fontFamily)
        assertNull(result[0].weight)
        assertNull(result[0].style)
    }

    @Test
    fun singleRuleWithAllFields() {
        val toml = """
            [[rules]]
            scope = "comment"
            [rules.font]
            family = "IBM Plex Mono"
            weight = "bold"
            style = "italic"
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(1, result.size)
        assertEquals("comment", result[0].scope)
        assertEquals("IBM Plex Mono", result[0].fontFamily)
        assertEquals("bold", result[0].weight)
        assertEquals("italic", result[0].style)
    }

    @Test
    fun nestedRulesFontSyntax() {
        val toml = """
            [[rules]]
            scope = "keyword"
            [rules.font]
            family = "Maple Mono"
            weight = "semi-bold"
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(1, result.size)
        assertEquals("keyword", result[0].scope)
        assertEquals("Maple Mono", result[0].fontFamily)
        assertEquals("semi-bold", result[0].weight)
    }

    @Test
    fun multipleRulesBlocks() {
        val toml = """
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
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(3, result.size)
        assertEquals("keyword", result[0].scope)
        assertEquals("comment", result[1].scope)
        assertEquals("string", result[2].scope)
        assertEquals("A", result[0].fontFamily)
        assertEquals("B", result[1].fontFamily)
        assertEquals("C", result[2].fontFamily)
    }

    @Test
    fun trailingWhitespaceAndBlankLines() {
        val toml = """
            version = 1


            [[rules]]
            scope = "keyword"
            [rules.font]
            family = "Fira Code"


        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(1, result.size)
        assertEquals("keyword", result[0].scope)
    }

    @Test
    fun commentsAreIgnored() {
        val toml = """
            # Top-level comment
            [[rules]]
            scope = "keyword"
            # Font section comment
            [rules.font]
            family = "Fira Code"
            # weight = "bold"
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(1, result.size)
        assertNull(result[0].weight)
    }

    @Test
    fun invalidInputReturnsEmptyList() {
        val result = PolyfontConfigParser.parse("{{invalid toml garbage}}")
        assertTrue(result.isEmpty())
    }

    @Test
    fun weightEdgeCaseEmpty() {
        val toml = """
            [[rules]]
            scope = "keyword"
            [rules.font]
            family = "Fira Code"
            weight = ""
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(1, result.size)
        assertEquals("", result[0].weight)
    }

    @Test
    fun weightEdgeCaseUnknownValue() {
        val toml = """
            [[rules]]
            scope = "keyword"
            [rules.font]
            family = "Fira Code"
            weight = "ultra-bold"
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(1, result.size)
        assertEquals("ultra-bold", result[0].weight)
    }

    @Test
    fun styleEdgeCaseEmpty() {
        val toml = """
            [[rules]]
            scope = "keyword"
            [rules.font]
            family = "Fira Code"
            style = ""
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(1, result.size)
        assertEquals("", result[0].style)
    }

    @Test
    fun differentTableClosesRuleContext() {
        val toml = """
            [[rules]]
            scope = "keyword"
            [rules.font]
            family = "A"
            [default]
            family = "Fira Code"
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(1, result.size)
        assertEquals("keyword", result[0].scope)
    }

    @Test
    fun scopeWithoutFontOmitsRule() {
        val toml = """
            [[rules]]
            scope = "keyword"
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertTrue(result.isEmpty())
    }

    @Test
    fun fontWithoutScopeOmitsRule() {
        val toml = """
            [[rules]]
            [rules.font]
            family = "Fira Code"
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertTrue(result.isEmpty())
    }

    @Test
    fun consecutiveRulesWithSameFontFamily() {
        val toml = """
            [[rules]]
            scope = "keyword"
            [rules.font]
            family = "Mono"
            [[rules]]
            scope = "keyword.operator"
            [rules.font]
            family = "Mono"
        """.trimIndent()
        val result = PolyfontConfigParser.parse(toml)
        assertEquals(2, result.size)
        assertEquals("Mono", result[0].fontFamily)
        assertEquals("Mono", result[1].fontFamily)
    }
}
