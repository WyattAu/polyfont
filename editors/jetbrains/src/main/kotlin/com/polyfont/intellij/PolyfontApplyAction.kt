package com.polyfont.intellij

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.editor.colors.EditorColorsManager
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.LocalFileSystem
import java.awt.Color
import java.io.File

class PolyfontApplyAction : AnAction() {

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val configRules = loadRulesFromConfig(project)
        if (configRules.isEmpty() && PolyfontSettings.getInstance().fontRules.isEmpty()) {
            notify(project, "No font rules found in .polyfont.toml", NotificationType.WARNING)
            return
        }

        val settingsRules = PolyfontSettings.getInstance().fontRules.map { (scope, fontFamily) ->
            PolyfontConfigParser.Rule(scope, fontFamily)
        }
        val merged = configRules + settingsRules
        val scheme = EditorColorsManager.getInstance().globalScheme
        var applied = 0

        // TODO: Per-scope font families are not supported by TextAttributes (only bold/italic flags).
        //  To apply actual different fonts per scope, a custom HighlightVisitor or EditorNotificationProvider
        //  would be needed. For now, we set the font type (plain/bold/italic/bold-italic) based on weight/style.
        for (rule in merged) {
            val key = TextAttributesKey.createTextAttributesKey("POLYFONT_${rule.scope}")
            val fontType = computeFontType(rule.weight, rule.style)
            val attrs = TextAttributes(
                scheme.defaultForeground,
                null,
                null,
                null,
                fontType,
            )
            val existing = scheme.getAttributes(key)
            if (existing != null) {
                attrs.foregroundColor = existing.foregroundColor
            }
            scheme.setAttributes(key, attrs)
            applied++
        }

        notify(project, "Applied $applied polyfont font rule(s)", NotificationType.INFORMATION)
    }

    private fun computeFontType(weight: String?, style: String?): Int {
        val bold = weight != null && weight.lowercase() in listOf("bold", "700", "800", "900")
        val italic = style != null && style.lowercase() in listOf("italic", "oblique")
        return when {
            bold && italic -> 3
            bold -> 1
            italic -> 2
            else -> 0
        }
    }

    private fun loadRulesFromConfig(project: Project): List<PolyfontConfigParser.Rule> {
        val baseDir = project.basePath ?: return emptyList()
        val configFile = File(baseDir, ".polyfont.toml")
        if (!configFile.exists()) return emptyList()
        return PolyfontConfigParser.parse(configFile.readText())
    }

    private fun notify(project: Project, content: String, type: NotificationType) {
        NotificationGroupManager.getInstance()
            .getNotificationGroup("Polyfont Notifications")
            .createNotification(content, type)
            .notify(project)
    }
}
