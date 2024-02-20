import * as vscode from 'vscode';
import { Wasmer, init, Directory, initializeLogger, setWorkerUrl, setDefaultWorkerUrl } from "@wasmer/sdk";
import WasmerSDKPath from "../../dist/wasmer_js_bg.wasm";
import { WasmPseudoTerminal } from './terminal';

export async function activate(context: vscode.ExtensionContext): Promise<void> {

	const workerUrl = vscode.Uri.joinPath(context.extensionUri, "/dist/wasmer_js.js");
	const workerWasmUrl = vscode.Uri.joinPath(context.extensionUri, "/dist/wasmer_js_bg.wasm");
	const wasmerBinary = await vscode.workspace.fs.readFile(workerWasmUrl);

	await init(wasmerBinary);

	setWorkerUrl(workerUrl.toString());
	initializeLogger("warn");

	const fs = new Directory();
	const pkg = await Wasmer.fromRegistry("sharrattj/bash");

    context.subscriptions.push(
        vscode.commands.registerCommand("wasmer-bash.openTerminal", async function() {
			const terminal = vscode.window.createTerminal({
				name: "wasmer-bash",
				pty: await WasmPseudoTerminal.createWasmPseudoTerminal(pkg, fs)
			});
			terminal.show();
		})
    );
}
