import * as vscode from 'vscode';
import { Wasmer, init, WasiFS, initializeLogger, setWorkerUrl, setDefaultWorkerUrl } from "@wasmer/sdk";
import { importVSCode } from 'vscode-interop';
import { WasmPseudoTerminal } from './terminal';

export async function activate(context: vscode.ExtensionContext): Promise<void> {

	const workerUrl = vscode.Uri.joinPath(context.extensionUri, "/dist/wasmer_js.js");
	const workerWasmUrl = vscode.Uri.joinPath(context.extensionUri, "/dist/wasmer_js_bg.wasm");
	const wasmerBinary = await vscode.workspace.fs.readFile(workerWasmUrl);

	await importVSCode();
	await init(wasmerBinary);

	setWorkerUrl(workerUrl.toString());
	initializeLogger("warn");

	const fs = new WasiFS();
	const pkg = await Wasmer.fromRegistry("sharrattj/bash");

    context.subscriptions.push(
        vscode.commands.registerCommand("wasmer-bash.openTerminal", async function() {
			const terminal = vscode.window.createTerminal({
				name: "wasmer-bash",
				pty: await WasmPseudoTerminal.createWasmPseudoTerminal(pkg, fs.clone())
			});
			terminal.show();
		})
    );

	if (vscode.workspace.workspaceFolders) {
		for (const added of vscode.workspace.workspaceFolders) {
			const folderName = added.name || added.index;
			console.log(`mount /workspace/${folderName}`);
			fs.mount(added.uri, "/" + folderName);
		}
	}

	vscode.workspace.onDidChangeWorkspaceFolders(e => {
		for (const added of e.added) {
			const folderName = added.name || added.index;
			console.log(`mount /workspace/${folderName}`);
			fs.mount(added.uri, "/" + folderName);
		}

		for (const removed of e.removed) {
			const folderName = removed.name || removed.index;
			console.log(`unmount /workspace/${folderName}`);
			fs.unmount("/" + folderName);
		}
	});
}
