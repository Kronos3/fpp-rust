# FPP LSP Parser

This is the second of the two parsers in FPP. This parser's responsibility
is to provide a syntax tree (_not_ abstract syntax tree) that losslessly
represents the source text.

This implementation is based on Rust Analyzer's Rowan implementation which
implements a Red/Green Syntax tree. Much of the basic helper/builder code
is copied directly from the Rust Analyzer source. Nearly all the code in the
top level of the crate is directly from Rust Analyzer. The FPP parsing code
can be found in `src/grammar`.

This parser will interface with the language server to provide semantic
highlighting, edits, formatting, auto-complete etc.
