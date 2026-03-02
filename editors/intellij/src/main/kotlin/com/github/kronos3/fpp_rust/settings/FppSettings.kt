package com.github.kronos3.fpp_rust.settings

import com.github.kronos3.fpp_rust.reloadProject
import com.github.kronos3.fpp_rust.restartFppServerAsync
import com.github.kronos3.fpp_rust.util.Version
import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.Storage
import com.intellij.openapi.components.StoragePathMacros
import com.intellij.openapi.project.Project

enum class LspConfigurationType {
    Disabled, Auto, Manual,
}

sealed class FppProject {
    data class LocsFile(val uri: String) : FppProject()
    data class EntireWorkspace(val uri: String) : FppProject()
}

@Service(Service.Level.PROJECT)
@State(name = "FppServiceSettings", storages = [Storage(StoragePathMacros.WORKSPACE_FILE)])
class FppSettings(val project: Project) : PersistentStateComponent<FppSettings.State> {
    data class State(
        val lspVersion: String? = null,
        val lspConfigurationType: LspConfigurationType = LspConfigurationType.Auto,
        val lspPath: String = "",
        val project: FppProject? = null
    )

    private var internalState: State = State()

    var lspPath: String
        get() = internalState.lspPath
        set(value) {
            internalState = internalState.copy(
                lspPath = value
            )
        }

    var lspVersion: Version?
        get() = internalState.lspVersion?.let { Version.parse(it) }
        set(value) {
            internalState = internalState.copy(
                lspVersion = value?.let { value.toString() }
            )
        }

    var lspConfigurationType: LspConfigurationType
        get() = internalState.lspConfigurationType
        set(value) {
            internalState = internalState.copy(
                lspConfigurationType = value
            )
        }

    override fun getState() = internalState
    override fun loadState(state: State) {
        val lspBinaryPathModified = state.lspPath != internalState.lspPath
        val projectModified = state.project != internalState.project
        internalState = state

        if (lspBinaryPathModified) restartFppServerAsync(project)
        if (projectModified) reloadProject(project)
    }

    companion object {
        fun getInstance(project: Project): FppSettings =
            project.getService(FppSettings::class.java)
    }
}