import * as lc from "vscode-languageclient";

export const reloadWorkspace = new lc.RequestType0<void, void>("fpp/reloadWorkspace");
export const setLocsWorkspace = new lc.RequestType<lc.URI, void, void>("fpp/setLocsWorkspace");
export const setFilesWorkspace = new lc.RequestType<lc.URI, void, void>("fpp/setFilesWorkspace");
