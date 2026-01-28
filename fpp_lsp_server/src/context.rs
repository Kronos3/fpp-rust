use fpp_core::{DiagnosticData, DiagnosticEmitter};
use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, Position, Range, Uri,
};
use rustc_hash::FxHashMap;
use std::str::FromStr;

#[derive(Clone, Default)]
pub struct LspDiagnosticsEmitter {
    pub diagnostics: FxHashMap<Uri, Vec<Diagnostic>>,
}

impl LspDiagnosticsEmitter {
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

fn span_data_to_range(span: &fpp_core::SpanData) -> Range {
    let file = span.file.upgrade().unwrap();
    let start = file.lines.line_col(span.start.into());
    let end = file.lines.line_col((span.start + span.length).into());

    Range::new(
        Position::new(start.line, start.col),
        Position::new(end.line, end.col),
    )
}

fn diagnostic_level_to_severity(level: fpp_core::Level) -> DiagnosticSeverity {
    match level {
        fpp_core::Level::Error => DiagnosticSeverity::ERROR,
        fpp_core::Level::Warning => DiagnosticSeverity::WARNING,
        fpp_core::Level::Note => DiagnosticSeverity::INFORMATION,
        fpp_core::Level::Help => DiagnosticSeverity::HINT,
        _ => DiagnosticSeverity::INFORMATION,
    }
}

impl DiagnosticEmitter for LspDiagnosticsEmitter {
    fn emit(&mut self, diagnostic: DiagnosticData) {
        let file = diagnostic.span.file.upgrade().unwrap();
        let uri = Uri::from_str(&file.uri).unwrap();
        let range = span_data_to_range(&diagnostic.span);

        let uri_c = uri.clone();
        let related_information = Some(
            diagnostic
                .children
                .into_iter()
                .map(|sub| {
                    let location = match sub.span {
                        None => Location::new(uri_c.clone(), range.clone()),
                        Some(span) => Location::new(uri_c.clone(), span_data_to_range(&span)),
                    };

                    DiagnosticRelatedInformation {
                        location,
                        message: sub.message,
                    }
                })
                .collect(),
        );

        let lsp_diag = Diagnostic {
            range,
            severity: Some(diagnostic_level_to_severity(diagnostic.level)),
            source: Some(uri.as_str().to_owned()),
            message: diagnostic.message,
            related_information,
            ..Diagnostic::default()
        };

        match self.diagnostics.get_mut(&uri) {
            None => {
                self.diagnostics.insert(uri, vec![lsp_diag]);
            }
            Some(c) => c.push(lsp_diag),
        }
    }
}
