package com.github.kronos3.fpp_rust

import com.intellij.openapi.options.Configurable
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.NlsContexts
import javax.swing.JComponent


/**
 * Provides controller functionality for application settings.
 */
internal class FppSettingsConfigurable(private val project: Project) : Configurable {
    private var mySettingsComponent: FppSettingsComponent? = null

    override fun getDisplayName(): @NlsContexts.ConfigurableName String {
        return "FPP: Settings"
    }

    override fun getPreferredFocusedComponent(): JComponent {
        return mySettingsComponent!!.preferredFocusedComponent
    }

    override fun createComponent(): JComponent? {
        mySettingsComponent = FppSettingsComponent()
        return mySettingsComponent!!.panel
    }

    override fun isModified(): Boolean {
        val state: FppSettings.State =
            requireNotNull(FppSettings.getInstance(project)).state
        return !mySettingsComponent?.lspBinaryPathText.equals(state.lspBinaryPath)
    }

    override fun apply() {
        val state: FppSettings.State =
            requireNotNull(FppSettings.getInstance(project)).state
        state.lspBinaryPath = mySettingsComponent!!.lspBinaryPathText
    }

    override fun reset() {
        val state: FppSettings.State =
            requireNotNull(FppSettings.getInstance(project)).state
        mySettingsComponent!!.lspBinaryPathText = state.lspBinaryPath
    }

    override fun disposeUIResources() {
        mySettingsComponent = null
    }
}

