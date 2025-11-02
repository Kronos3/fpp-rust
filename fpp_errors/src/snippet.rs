use annotate_snippets::{AnnotationKind, Element, Group, Snippet};
use fpp_core::{DiagnosticData, Level};

fn diagnostic_level<'a>(level: Level) -> annotate_snippets::Level<'a> {
    match level {
        Level::Error => annotate_snippets::Level::ERROR,
        Level::Warning => annotate_snippets::Level::WARNING,
        Level::Note => annotate_snippets::Level::NOTE,
        Level::Help => annotate_snippets::Level::HELP,
        _ => annotate_snippets::Level::INFO,
    }
}

pub(crate) fn diagnostic_to_snippet_group<'a>(diagnostic: &'a DiagnosticData) -> Group<'a> {
    (match &diagnostic.message.snippet {
        None => Group::with_title(
            diagnostic_level(diagnostic.message.level)
                .primary_title(diagnostic.message.message.clone()),
        ),
        Some(snippet) => Group::with_level(diagnostic_level(diagnostic.message.level)).element(
            Snippet::source(snippet.file_content)
                .line_start(snippet.line_offset)
                .path(snippet.file_path)
                .annotation(
                    AnnotationKind::Primary
                        .span(snippet.start..snippet.end)
                        .label(diagnostic.message.message.clone()),
                ),
        ),
    })
    .elements(diagnostic.children.iter().map(|child| {
        match &child.snippet {
            None => Element::Message(diagnostic_level(child.level).message(child.message.clone())),
            Some(snippet) => Snippet::source(snippet.file_content)
                .line_start(snippet.line_offset)
                .path(snippet.file_path)
                .annotation(
                    AnnotationKind::Primary
                        .span(snippet.start..snippet.end)
                        .label(child.message.clone()),
                )
                .into(),
        }
    }))
}
