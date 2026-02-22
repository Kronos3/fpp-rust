package com.github.kronos3.fpp_rust

import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.Storage
import com.intellij.openapi.components.StoragePathMacros
import com.intellij.openapi.project.Project

sealed class FppProject {
    data class LocsFile(val uri: String) : FppProject()
    data class EntireWorkspace(val uri: String) : FppProject()
}

@Service(Service.Level.PROJECT)
@State(name = "FppServiceSettings", storages = [Storage(StoragePathMacros.WORKSPACE_FILE)])
class FppSettings(val project: Project) : PersistentStateComponent<FppSettings.State> {
    class State {
        var lspBinaryPath: String = ""
        var project: FppProject? = null
    }

    private var myState = State()

    override fun getState() = myState
    override fun loadState(state: State) {
        val lspBinaryPathModified = state.lspBinaryPath != myState.lspBinaryPath
        val projectModified = state.project != myState.project
        myState = state

        if (lspBinaryPathModified) restartFppServerAsync(project)
        if (projectModified) reloadProject(project)
    }

    companion object {
        fun getInstance(project: Project): FppSettings =
            project.getService(FppSettings::class.java)
    }
}