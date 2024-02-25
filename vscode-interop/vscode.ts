import { FileSystemError, FileSystem } from "vscode";

let code: typeof import("vscode") | undefined;

export async function importVSCode() {
    return code || (code = await import("vscode"));
}

export function getWorkspaceFs(): FileSystem | undefined {
    return code?.workspace.fs;
}

export function getFileSystemError(): typeof FileSystemError | undefined {
    return code?.FileSystemError;
}
