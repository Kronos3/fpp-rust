package com.github.kronos3.fpp_rust

import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBTextField
import com.intellij.util.ui.FormBuilder
import javax.swing.JComponent
import javax.swing.JPanel

/**
 * Supports creating and managing a [JPanel] for the Settings Dialog.
 */
class FppSettingsComponent {
    private val myMainPanel: JPanel?
    private val myLspBinaryPathText = JBTextField()

    init {
        myMainPanel = FormBuilder.createFormBuilder()
            .addLabeledComponent(JBLabel("LSP binary path:"), myLspBinaryPathText, 1, false)
            .addComponentFillVertically(JPanel(), 0)
            .getPanel()
    }

    val panel: JPanel?
        get() = myMainPanel

    val preferredFocusedComponent: JComponent
        get() = myLspBinaryPathText

    var lspBinaryPathText: String
        get() = myLspBinaryPathText.getText()
        set(newText) {
            myLspBinaryPathText.setText(newText)
        }
}