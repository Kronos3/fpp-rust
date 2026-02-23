use crate::cursor::Cursor;
use crate::token::Token;
use fpp_core::SourceFile;
use fpp_lexer::TokenKind::*;
use fpp_lexer::{KeywordKind, TokenKind};

struct Index(usize);

impl Index {
    fn next(&mut self) -> usize {
        let out = self.0;
        self.0 += 1;
        out
    }
}

fn lex(content: &str) -> Vec<Token> {
    let mut diagnostics_str = vec![];
    let mut ctx =
        fpp_core::CompilerContext::new(fpp_errors::WriteEmitter::new(&mut diagnostics_str));

    fpp_core::run(&mut ctx, || {
        let file = SourceFile::new("<stdin>", content.to_string());
        let mut cursor = Cursor::new(file, content, None);

        let mut out = vec![];
        loop {
            match cursor.next() {
                None => break,
                Some(tok) => {
                    out.push(tok);
                }
            }
        }

        out
    })
}

fn assert_token_eq(token: &Token, kind: TokenKind, text: &str) {
    assert_eq!(token.kind(), kind);
    assert_eq!(token.text(), text);
}

#[test]
fn skip_whitespace() {
    let tokens = lex("   ");
    assert_eq!(tokens.len(), 0)
}

#[test]
fn comment() {
    let tokens = lex(r#" # comment

    # more comments

    "#);
    assert_token_eq(&tokens[0], Eol, "");
    assert_eq!(tokens.len(), 1);
}

#[test]
fn eat_newlines() {
    let tokens = lex(r#"
    
    "#);
    assert_eq!(tokens.len(), 1);
    let mut idx = Index(0);
    assert_token_eq(&tokens[idx.next()], Eol, "");
}

#[test]
fn literals() {
    let tokens = lex(
        r#"12 1.23 0x10 0x1AEF 001 1e30 10.3e3 .3e3 "" "string \"" """
    a multiline literal string with \"\"\" some \escapes " "
    """ """""" "#,
    );
    let mut idx = Index(0);
    assert_token_eq(&tokens[idx.next()], LiteralInt, "12");
    assert_token_eq(&tokens[idx.next()], LiteralFloat, "1.23");
    assert_token_eq(&tokens[idx.next()], LiteralInt, "0x10");
    assert_token_eq(&tokens[idx.next()], LiteralInt, "0x1AEF");
    assert_token_eq(&tokens[idx.next()], LiteralInt, "001");
    assert_token_eq(&tokens[idx.next()], LiteralFloat, "1e30");
    assert_token_eq(&tokens[idx.next()], LiteralFloat, "10.3e3");
    assert_token_eq(&tokens[idx.next()], LiteralFloat, ".3e3");
    assert_token_eq(&tokens[idx.next()], LiteralString, "");
    assert_token_eq(&tokens[idx.next()], LiteralString, "string \\\"");

    assert_token_eq(
        &tokens[idx.next()],
        LiteralMultilineString { indent: 4 },
        "\na multiline literal string with \\\"\\\"\\\" some \\escapes \" \"\n",
    );
    assert_token_eq(
        &tokens[idx.next()],
        LiteralMultilineString { indent: 0 },
        "",
    );
    assert_eq!(tokens.len(), 12);
}

#[test]
fn annotations() {
    let tokens = lex(r#"@ Pre annotation
        Some Identifiers @< Post annotation"#);

    assert_eq!(tokens.len(), 4);
    let mut idx = Index(0);
    assert_token_eq(&tokens[idx.next()], PreAnnotation, "Pre annotation");
    assert_token_eq(&tokens[idx.next()], Identifier, "Some");
    assert_token_eq(&tokens[idx.next()], Identifier, "Identifiers");
    assert_token_eq(&tokens[idx.next()], PostAnnotation, "Post annotation");
}

#[test]
fn identifiers_and_keywords() {
    let tokens = lex(r#"Ident _underscope_start with_numbers01_asd yellow $yellow action every"#);

    assert_eq!(tokens.len(), 7);
    let mut idx = Index(0);
    assert_token_eq(&tokens[idx.next()], Identifier, "Ident");
    assert_token_eq(&tokens[idx.next()], Identifier, "_underscope_start");
    assert_token_eq(&tokens[idx.next()], Identifier, "with_numbers01_asd");
    assert_token_eq(&tokens[idx.next()], Keyword(KeywordKind::Yellow), "");
    assert_token_eq(&tokens[idx.next()], Identifier, "yellow");
    assert_token_eq(&tokens[idx.next()], Keyword(KeywordKind::Action), "");
    assert_token_eq(&tokens[idx.next()], Keyword(KeywordKind::Every), "");
}

#[test]
fn escape_newline() {
    let tokens = lex(r#"escaped \
    newline"#);
    assert_eq!(tokens.len(), 2);
    let mut idx = Index(0);
    assert_token_eq(&tokens[idx.next()], Identifier, "escaped");
    assert_token_eq(&tokens[idx.next()], Identifier, "newline");
}

#[test]
fn invalid_tokens() {
    let tokens = lex(r#"1ee 	 $ 1e1e "
    ""#);
    assert_eq!(tokens.len(), 5);
    let mut idx = Index(0);
    assert_token_eq(&tokens[idx.next()], LiteralFloat, "1ee");
    assert_token_eq(&tokens[idx.next()], LiteralFloat, "1e1e");
    assert_token_eq(&tokens[idx.next()], LiteralString, "");
    assert_token_eq(&tokens[idx.next()], Eol, "");
    assert_token_eq(&tokens[idx.next()], LiteralString, "");

    let tokens = lex(r#"""" asdaldkasl"#);
    assert_eq!(tokens.len(), 1);
    // assert_token_eq(&tokens[0], Error("unclosed multi-line string literal"), "");
}

#[test]
fn escape_newline_error() {
    let tokens = lex(r#"escaped \hello
    newline"#);
    assert_eq!(tokens.len(), 2);
    let mut idx = Index(0);
    assert_token_eq(&tokens[idx.next()], Identifier, "escaped");
    // assert_token_eq(
    //     &tokens[idx.next()],
    //     Error("Non whitespace character illegal after line continuation"),
    //     "",
    // );
    assert_token_eq(&tokens[idx.next()], Identifier, "newline");
}

#[test]
fn symbols() {
    let tokens = lex(r#": . ,

        = ()

        ) {}

        } []

        ] -> - + ; / *

        1

        )1

        }1

        ]"#);

    assert_eq!(tokens.len(), 25);
    let mut idx = Index(0);
    assert_token_eq(&tokens[idx.next()], Colon, "");
    assert_token_eq(&tokens[idx.next()], Dot, "");
    assert_token_eq(&tokens[idx.next()], Comma, "");
    assert_token_eq(&tokens[idx.next()], Equals, "");
    assert_token_eq(&tokens[idx.next()], LeftParen, "");
    assert_token_eq(&tokens[idx.next()], RightParen, "");
    assert_token_eq(&tokens[idx.next()], RightParen, "");
    assert_token_eq(&tokens[idx.next()], LeftCurly, "");
    assert_token_eq(&tokens[idx.next()], RightCurly, "");
    assert_token_eq(&tokens[idx.next()], RightCurly, "");
    assert_token_eq(&tokens[idx.next()], LeftSquare, "");
    assert_token_eq(&tokens[idx.next()], RightSquare, "");
    assert_token_eq(&tokens[idx.next()], RightSquare, "");
    assert_token_eq(&tokens[idx.next()], RightArrow, "");
    assert_token_eq(&tokens[idx.next()], Minus, "");
    assert_token_eq(&tokens[idx.next()], Plus, "");
    assert_token_eq(&tokens[idx.next()], Semi, "");
    assert_token_eq(&tokens[idx.next()], Slash, "");
    assert_token_eq(&tokens[idx.next()], Star, "");
    assert_token_eq(&tokens[idx.next()], LiteralInt, "1");
    assert_token_eq(&tokens[idx.next()], RightParen, "");
    assert_token_eq(&tokens[idx.next()], LiteralInt, "1");
    assert_token_eq(&tokens[idx.next()], RightCurly, "");
    assert_token_eq(&tokens[idx.next()], LiteralInt, "1");
    assert_token_eq(&tokens[idx.next()], RightSquare, "");
}
