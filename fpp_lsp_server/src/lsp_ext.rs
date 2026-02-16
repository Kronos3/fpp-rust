use lsp_types::request::Request;
use serde::{Deserialize, Serialize};

pub enum ReloadWorkspace {}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UriRequest {
    pub uri: lsp_types::Uri
}

impl Request for ReloadWorkspace {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "fpp/reloadWorkspace";
}

pub enum SetLocsWorkspace {}

impl Request for SetLocsWorkspace {
    type Params = UriRequest;
    type Result = ();
    const METHOD: &'static str = "fpp/setLocsWorkspace";
}

pub enum SetFilesWorkspace {}

impl Request for SetFilesWorkspace {
    type Params = UriRequest;
    type Result = ();
    const METHOD: &'static str = "fpp/setFilesWorkspace";
}
