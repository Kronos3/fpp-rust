use lsp_types::request::Request;

pub enum ReloadWorkspace {}

impl Request for ReloadWorkspace {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "fpp/reloadWorkspace";
}

pub enum SetLocsWorkspace {}

impl Request for SetLocsWorkspace {
    type Params = lsp_types::Uri;
    type Result = ();
    const METHOD: &'static str = "fpp/setLocsWorkspace";
}

pub enum SetFilesWorkspace {}

impl Request for SetFilesWorkspace {
    type Params = lsp_types::Uri;
    type Result = ();
    const METHOD: &'static str = "fpp/setFilesWorkspace";
}
