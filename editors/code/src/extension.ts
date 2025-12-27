import * as vscode from "vscode";
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from "vscode-languageclient/node";

import * as Settings from "./settings";

let extension: FppExtension;

class FppExtension implements vscode.Disposable {
    private subscriptions: vscode.Disposable[];
    private outputChannel: vscode.OutputChannel;
    private traceOutputChannel: vscode.OutputChannel;

    client?: LanguageClient;

    constructor() {
        this.outputChannel = vscode.window.createOutputChannel("FPP", { log: true });
        this.traceOutputChannel = vscode.window.createOutputChannel("FPP Trace", { log: true });

        this.subscriptions = [
            Settings.onLspServerPathChanged(() => {
                this.initializeClient();
            }),
            this.outputChannel,
            this.traceOutputChannel,
        ];

    }

    async initializeClient() {
        try {
            await this.deactivate();
        } catch (e) {
            vscode.window.showErrorMessage(`Failed to deactivate old language server: ${e}`);
        }

        const serverPath = Settings.serverPath();
        if (!serverPath) {
            // TODO(tumbar) Add a button to open up settings
            vscode.window.showErrorMessage("No FPP server path set");
            return;
        }

        const serverOptions: ServerOptions = {
            run: {
                command: serverPath,
                transport: TransportKind.stdio,
            },
            debug: {
                command: serverPath,
                transport: TransportKind.stdio,
            },
        };

        const clientOptions: LanguageClientOptions = {
            documentSelector: [{ scheme: "file", language: "fpp" }],
            diagnosticCollectionName: "fpp",
            synchronize: {
                fileEvents: [
                    vscode.workspace.createFileSystemWatcher("**/*.fpp"),
                    vscode.workspace.createFileSystemWatcher("**/*.fppi"),
                ],
            },
            outputChannel: this.outputChannel,
            traceOutputChannel: this.traceOutputChannel,
        };

        try {
            this.client = new LanguageClient("fpp", "F Prime Prime", serverOptions, clientOptions);
            await this.client.start();
            vscode.window.showInformationMessage("FPP Server activated");
        } catch (e) {
            vscode.window.showErrorMessage(`Failed to start language server: ${e}`);
        }
    }

    async deactivate() {
        await this.client?.stop();
        this.client = undefined;
    }

    dispose() {
        for (const s of this.subscriptions) {
            s.dispose();
        }
    }
}

export async function activate(context: vscode.ExtensionContext) {
    extension = new FppExtension();
    context.subscriptions.push(
        extension,
        vscode.commands.registerCommand("fpp.restartLsp", async () => {
            await extension.initializeClient();
        })
    );

    await extension.initializeClient();
}

export function deactivate(): Thenable<void> | undefined {
    if (!extension) {
        return undefined;
    }
    return extension.deactivate();
}
