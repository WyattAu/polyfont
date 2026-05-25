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
        val rules = loadRulesFromConfig(project)
        if (rules.isEmpty()) {
            notify(project, "No font rules found in .polyfont.toml", NotificationType.WARNING)
            return
        }

        val merged = rules + PolyfontSettings.getInstance().fontRules
        val scheme = EditorColorsManager.getInstance().globalScheme
        var applied = 0

        for ((scope, fontFamily) in merged) {
            val key = TextAttributesKey.createTextAttributesKey("POLYFONT_$scope")
            val attrs = TextAttributes(
                scheme.defaultForeground,
                null,
                null,
                null,
                scheme.getFontPreferences().getFontFamily().let { 0 },
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

    private fun loadRulesFromConfig(project: Project): Map<String, String> {
        val baseDir = project.basePath ?: return emptyMap()
        val configFile = File(baseDir, ".polyfont.toml")
        if (!configFile.exists()) return emptyMap()
        return PolyfontConfigParser.parse(configFile.readText())
    }

    private fun notify(project: Project, content: String, type: NotificationType) {
        NotificationGroupManager.getInstance()
            .getNotificationGroup("Polyfont Notifications")
            .createNotification(content, type)
            .notify(project)
    }
}
