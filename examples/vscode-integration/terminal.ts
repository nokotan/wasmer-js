import * as vscode from 'vscode';
import { Pseudoterminal, Event, EventEmitter, TerminalDimensions, workspace, Uri } from 'vscode';
import { Wasmer, Instance, WasiFS, Directory } from '@wasmer/sdk';

export class WasmPseudoTerminal implements Pseudoterminal {
	onDidWrite: Event<string>;
	onDidClose: Event<number>;

	private m_rows: number = 30;
	private m_cols: number = 12;

	private stdin?: WritableStreamDefaultWriter<Uint8Array>;
	private encoder = new TextEncoder();
	private decoder = new TextDecoder();

	private constructor(private session: Instance, private writeEmitter: EventEmitter<string>, private closeEmitter: EventEmitter<number>) {	
		this.stdin = session.stdin?.getWriter();

		session.stdout.pipeTo(new WritableStream<Uint8Array>({ write: chunk => this.write(chunk) }));
		session.stderr.pipeTo(new WritableStream<Uint8Array>({ write: chunk => this.write(chunk) }));

		this.onDidWrite = this.writeEmitter.event;
		this.onDidClose = this.closeEmitter.event;
	}

	static async createWasmPseudoTerminal(bootCommand: Wasmer, fs: WasiFS) {
		const instance = await bootCommand.entrypoint!.run({
			mount: {
				"/workspace": fs as unknown as Directory
			}
		});
		return new WasmPseudoTerminal(instance, new vscode.EventEmitter<string>(), new vscode.EventEmitter<number>());
	}

    private onDimensionChangedCallback?: (dimension: TerminalDimensions) => void;

    async open(initialDimensions: TerminalDimensions | undefined) {
		if (initialDimensions) {
			this.m_rows = initialDimensions.rows;
			this.m_cols = initialDimensions.columns;
		}
    }

    close(): void {
		this.session?.free();
	}

	write(data: Uint8Array) {
		const text = this.decoder.decode(data);
		this.writeEmitter.fire(text.replace(/\n/g, "\r\n"));
	}

	handleInput(data: string) {
		this.stdin?.write(this.encoder.encode(data));
	}

	onClose(exitCode: number) {
		this.closeEmitter.fire(exitCode);
	}

	setDimensions(dimensions: TerminalDimensions): void {
		this.m_rows = dimensions.rows;
		this.m_cols = dimensions.columns;
		this.onDimensionChangedCallback?.call(this, dimensions);
	}

    onDimensionChanged(callback: (dimension: TerminalDimensions) => void) {
        this.onDimensionChangedCallback = callback;
    }

	get rows() {
		return this.m_rows;
	}

	get cols() {
		return this.m_cols;
	}
}