import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';
import * as ext from "./lsp_ext";

export interface WorkspaceFileScanner {
    label(): string;

    scan(
        client: LanguageClient,
        progress: vscode.Progress<{ message?: string; increment?: number }>,
        token: vscode.CancellationToken,
    ): Promise<void>;
}

export class LocsFileScanner implements WorkspaceFileScanner {
    constructor(readonly locsFile: vscode.Uri) { }

    label(): string {
        return vscode.workspace.asRelativePath(this.locsFile);
    }

    async scan(
        client: LanguageClient,
        progress: vscode.Progress<{ message?: string; increment?: number }>,
        token: vscode.CancellationToken,
    ): Promise<void> {
        // Read the locsFile file
        progress.report({
            message: "Loading FPP workspace through locs file",
        });

        await client.sendRequest(ext.setLocsWorkspace, { uri: this.locsFile.toString() });
    }
}


export class EntireWorkspaceScanner implements WorkspaceFileScanner {
    constructor() { }

    label(): string {
        return "Workspace";
    }

    async scan(client: LanguageClient, progress: vscode.Progress<{ message?: string; increment?: number; }>, token: vscode.CancellationToken): Promise<void> {
        if (!vscode.workspace.workspaceFolders) {
            // No workspace is loaded, cannot load FPP project
            return;
        }

        // Glob the entire workspace
        progress.report({
            message: "Scanning for .fpp files in workspace",
            increment: 0
        });

        await client.sendRequest(ext.setFilesWorkspace, {
            uri: vscode.workspace.workspaceFolders[0].uri.toString()
        });
    }
}
