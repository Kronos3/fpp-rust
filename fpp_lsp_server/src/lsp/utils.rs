use std::{mem, ops::Range};

use fpp_core::{LineCol, LineIndex, TextRange, TextSize, WideLineCol};

use crate::lsp::capabilities::PositionEncoding;

pub(crate) fn offset(
    line_index: &LineIndex,
    enc: PositionEncoding,
    position: lsp_types::Position,
) -> anyhow::Result<TextSize> {
    let line_col = match enc {
        PositionEncoding::Utf8 => LineCol {
            line: position.line,
            col: position.character,
        },
        PositionEncoding::Wide(enc) => {
            let line_col = WideLineCol {
                line: position.line,
                col: position.character,
            };
            line_index
                .to_utf8(enc, line_col)
                .ok_or_else(|| anyhow::anyhow!("Invalid wide col offset"))?
        }
    };
    let line_range = line_index.line(line_col.line).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid offset {line_col:?} (line index length: {:?})",
            line_index.len()
        )
    })?;
    let col = TextSize::from(line_col.col);
    let clamped_len = col.min(line_range.len());
    // FIXME: The cause for this is likely our request retrying. Commented out as this log is just too chatty and very easy to trigger.
    // if clamped_len < col {
    //     tracing::error!(
    //         "Position {line_col:?} column exceeds line length {}, clamping it",
    //         u32::from(line_range.len()),
    //     );
    // }
    Ok(line_range.start() + clamped_len)
}

pub(crate) fn text_range(
    line_index: &LineIndex,
    enc: PositionEncoding,
    range: lsp_types::Range,
) -> anyhow::Result<TextRange> {
    let start = offset(line_index, enc, range.start)?;
    let end = offset(line_index, enc, range.end)?;
    match end < start {
        true => Err(anyhow::anyhow!("Invalid Range")),
        false => Ok(TextRange::new(start, end)),
    }
}

pub(crate) fn apply_document_changes(
    encoding: PositionEncoding,
    file_contents: String,
    mut content_changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
) -> String {
    // If at least one of the changes is a full document change, use the last
    // of them as the starting point and ignore all previous changes.
    let (mut text, content_changes) = match content_changes
        .iter()
        .rposition(|change| change.range.is_none())
    {
        Some(idx) => {
            let text = mem::take(&mut content_changes[idx].text);
            (text, &content_changes[idx + 1..])
        }
        None => (file_contents, &content_changes[..]),
    };
    if content_changes.is_empty() {
        return text;
    }

    let mut line_index = fpp_core::LineIndex::new(&text);

    // The changes we got must be applied sequentially, but can cross lines so we
    // have to keep our line index updated.
    // Some clients (e.g. Code) sort the ranges in reverse. As an optimization, we
    // remember the last valid line in the index and only rebuild it if needed.
    // The VFS will normalize the end of lines to `\n`.
    let mut index_valid = !0u32;
    for change in content_changes {
        // The None case can't happen as we have handled it above already
        if let Some(range) = change.range {
            if index_valid <= range.end.line {
                line_index = fpp_core::LineIndex::new(&text);
            }
            index_valid = range.start.line;
            if let Ok(range) = text_range(&line_index, encoding, range) {
                text.replace_range(Range::<usize>::from(range), &change.text);
            }
        }
    }
    text
}
