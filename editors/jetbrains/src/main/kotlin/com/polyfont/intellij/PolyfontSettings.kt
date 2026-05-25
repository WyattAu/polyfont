package com.polyfont.intellij

import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.openapi.options.Configurable
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.table.JBTable
import com.intellij.util.xmlb.XmlSerializerUtil
import javax.swing.JButton
import javax.swing.JPanel
import javax.swing.JTable
import javax.swing.JTextField
import javax.swing.table.DefaultTableModel

@State(
    name = "com.polyfont.intellij.PolyfontSettings",
    storages = [Storage("polyfont.xml")]
)
class PolyfontSettings : PersistentStateComponent<PolyfontSettings> {
    var fontRules: Map<String, String> = emptyMap()

    override fun getState(): PolyfontSettings = this

    override fun loadState(state: PolyfontSettings) {
        XmlSerializerUtil.copyBean(state, this)
    }

    companion object {
        fun getInstance(): PolyfontSettings =
            com.intellij.openapi.application.ApplicationManager.getApplication().getService(PolyfontSettings::class.java)
    }
}

class PolyfontSettingsComponent {
    private val panel = JPanel()
    private val tableModel = DefaultTableModel(arrayOf("Scope", "Font Family"), 0)
    private val table: JTable = JBTable(tableModel)
    private val scopeField = JTextField(20)
    private val fontField = JTextField(20)

    init {
        val scrollPane = JBScrollPane(table)
        val addPanel = JPanel()
        addPanel.add(JBLabel("Scope:"))
        addPanel.add(scopeField)
        addPanel.add(JBLabel("Font:"))
        addPanel.add(fontField)
        val addButton = JButton("Add Rule")
        addButton.addActionListener {
            val scope = scopeField.text.trim()
            val font = fontField.text.trim()
            if (scope.isNotEmpty() && font.isNotEmpty()) {
                tableModel.addRow(arrayOf(scope, font))
                scopeField.text = ""
                fontField.text = ""
            }
        }
        addPanel.add(addButton)

        val removeButton = JButton("Remove Selected")
        removeButton.addActionListener {
            val rows = table.selectedRows
            for (i in rows.sortedArray().reversed()) {
                tableModel.removeRow(i)
            }
        }
        addPanel.add(removeButton)

        panel.layout = java.awt.BorderLayout()
        panel.add(addPanel, java.awt.BorderLayout.NORTH)
        panel.add(scrollPane, java.awt.BorderLayout.CENTER)
    }

    fun getPanel(): JPanel = panel

    fun reset(rules: Map<String, String>) {
        tableModel.rowCount = 0
        rules.forEach { (scope, font) ->
            tableModel.addRow(arrayOf(scope, font))
        }
    }

    fun getRules(): Map<String, String> {
        val rules = mutableMapOf<String, String>()
        for (i in 0 until tableModel.rowCount) {
            val scope = tableModel.getValueAt(i, 0) as? String ?: continue
            val font = tableModel.getValueAt(i, 1) as? String ?: continue
            rules[scope] = font
        }
        return rules
    }
}

class PolyfontConfigurable : Configurable {
    private val settings = PolyfontSettings.getInstance()
    private val component = PolyfontSettingsComponent()

    override fun getDisplayName(): String = "Polyfont"

    override fun createComponent(): JPanel = component.getPanel()

    override fun isModified(): Boolean {
        return component.getRules() != settings.fontRules
    }

    override fun apply() {
        settings.fontRules = component.getRules()
    }

    override fun reset() {
        component.reset(settings.fontRules)
    }
}
