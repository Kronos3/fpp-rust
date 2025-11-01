use crate::parser::Parser;
use crate::{parse, ParseResult};
use fpp_core::SourceFile;
use std::fmt::Debug;

fn parse_str<T>(
    content: &str,
    entry: fn(&mut Parser) -> ParseResult<T>,
) -> (ParseResult<T>, fpp_core::CompilerContext) {
    let mut ctx = fpp_core::CompilerContext::new();
    let res = fpp_core::run(&mut ctx, || parse(SourceFile::from(content), entry))
        .expect("compiler error");

    (res, ctx)
}

fn parse_ast_serialize<T>(content: &str, entry: fn(&mut Parser) -> ParseResult<T>) -> String
where
    T: Debug,
{
    let (ast_res, mut ctx) = parse_str(content, entry);
    let ast = ast_res.expect("parsing failed");
    fpp_core::run(&mut ctx, || format!("{:?}", ast)).unwrap()
}

#[test]
fn component_passive_empty() {
    assert_eq!(
        parse_ast_serialize(
            r#"passive component C {

    }"#,
            |parser: &mut Parser| parser.def_component()
        ),
        "DefComponent { kind: Passive, name: Ident { data: \"C\", node_id: Span { start: <input>:1:19, end: <input>:1:20 } }, members: [], node_id: Span { start: <input>:1:1, end: <input>:3:7 } }"
    )
}
