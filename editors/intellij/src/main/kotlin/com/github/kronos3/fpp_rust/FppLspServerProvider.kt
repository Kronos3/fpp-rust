package com.github.kronos3.fpp_rust

import com.intellij.execution.ExecutionException
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.icons.AllIcons
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServer
import com.intellij.platform.lsp.api.LspServerManager
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor
import com.intellij.platform.lsp.api.lsWidget.LspServerWidgetItem
import com.intellij.util.containers.addIfNotNull
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import org.eclipse.lsp4j.jsonrpc.services.JsonRequest
import org.eclipse.lsp4j.services.LanguageServer
import java.util.concurrent.CompletableFuture

private class ReloadWorkspaceAction(
    private val lspServer: LspServer,
) : AnAction(
    "Reload Workspace",
    "Reload FPP workspace by scanning all .fpp files in IDE workspace",
    AllIcons.Actions.Refresh
), DumbAware {
    override fun actionPerformed(e: AnActionEvent): Unit = run { e.project?.let { project -> loadAllFiles(project, lspServer) } }
}

open class FppLspProjectWidgetItem(lspServer: LspServer, currentFile: VirtualFile?) : LspServerWidgetItem(
    lspServer, currentFile,
    settingsPageClass = FppSettingsConfigurable::class.java
) {


    override fun createWidgetInlineActions(): List<AnAction> {
        val actions = super.createWidgetInlineActions().toMutableList()
        actions.addIfNotNull(ReloadWorkspaceAction(lspServer))

        return actions
    }
}

internal class FppLspServerSupportProvider : LspServerSupportProvider {
    override fun fileOpened(
        project: Project,
        file: VirtualFile,
        serverStarter: LspServerSupportProvider.LspServerStarter
    ) {
        if (file.extension == "fpp" || file.extension == "fppi") {
            serverStarter.ensureServerStarted(FppLspServerDescriptor(project))
        }
    }

    override fun createLspServerWidgetItem(lspServer: LspServer, currentFile: VirtualFile?): LspServerWidgetItem =
        FppLspProjectWidgetItem(lspServer, currentFile)

    fun loadProject(project: Project) {
        val fppProject = FppSettings.getInstance(project).state.project
        if (fppProject != null) {

        }
    }
}

private class FppLspServerDescriptor(project: Project) : ProjectWideLspServerDescriptor(project, "fpp") {
    override fun isSupportedFile(file: VirtualFile) = file.extension == "fpp" || file.extension == "fppi"
    override fun createCommandLine() = run {
        val savedPath = FppSettings.getInstance(project).state.lspBinaryPath

        if (savedPath.isEmpty()) {
            throw ExecutionException("LSP binary path is not configured. Please check Settings.")
        }

        GeneralCommandLine(savedPath).withParameters("--stdio")
            .withEnvironment("RUST_BACKTRACE", "1")
    }

    override val lsp4jServerClass = FppLsp4jServer::class.java
}

data class UriRequest(val uri: String)

interface FppLsp4jServer : LanguageServer {
    @JsonRequest("fpp/reloadWorkspace")
    fun reloadWorkspace(params: Void): CompletableFuture<Void>

    @JsonRequest("fpp/setLocsWorkspace")
    fun setLocsWorkspace(params: UriRequest): CompletableFuture<Void>

    @JsonRequest("fpp/setFilesWorkspace")
    fun setFilesWorkspace(params: UriRequest): CompletableFuture<Void>
}

fun loadAllFiles(project: Project, lspServer: LspServer) {
    ApplicationManager.getApplication().invokeLater({
        println("loading all FPP files")
        println(project)
        println(project.projectFilePath)
        println(project.workspaceFile)
        println(project.basePath)

        GlobalScope.launch {
            lspServer.sendRequest { (it as FppLsp4jServer).setFilesWorkspace(UriRequest("file://${project.basePath}")) }
        }
    }, project.disposed)
}

fun reloadProject(project: Project) {
    ApplicationManager.getApplication().invokeLater({
        LspServerManager.getInstance(project).getServersForProvider(FppLspServerSupportProvider::class.java).map {
            val fppProject = FppSettings.getInstance(project).state.project
            if (fppProject != null) {
                when (fppProject) {
                    is FppProject.EntireWorkspace -> {
                        (it as FppLsp4jServer).setFilesWorkspace(UriRequest(fppProject.uri))
                    }

                    is FppProject.LocsFile -> {
                        (it as FppLsp4jServer).setLocsWorkspace(UriRequest(fppProject.uri))
                    }
                }

            }
        }
    }, project.disposed)
}

fun restartFppServerAsync(project: Project) {
    ApplicationManager.getApplication().invokeLater({
        LspServerManager.getInstance(project).stopAndRestartIfNeeded(FppLspServerSupportProvider::class.java)
    }, project.disposed)
}
