mod component;
mod expr;
mod module;
mod types;
mod state_machine;
mod topology;

use crate::token_set::TokenSet;
use crate::{parser::Parser, SyntaxKind, SyntaxKind::*};

pub(super) const MEMBER_RECOVERY_SET: TokenSet = TokenSet::new(&[
    EOL,
    SEMI,
    TYPE_KW,
    ARRAY_KW,
    ASYNC_KW,
    GUARDED_KW,
    SYNC_KW,
    OUTPUT_KW,
    COMPONENT_KW,
    INSTANCE_KW,
    CONSTANT_KW,
    ENUM_KW,
    INTERFACE_KW,
    MODULE_KW,
    PORT_KW,
    STATE_KW,
    STRUCT_KW,
    TOPOLOGY_KW,
    INCLUDE_KW,
    LOCATE_KW,
]);

fn name_r(p: &mut Parser<'_>, recovery: TokenSet) {
    if p.at(IDENT) {
        let m = p.start();
        p.bump(IDENT);
        m.complete(p, NAME);
    } else {
        p.err_recover("expected a name", recovery);
    }
}

fn name(p: &mut Parser<'_>) {
    name_r(p, TokenSet::EMPTY);
}

fn name_ref_r(p: &mut Parser<'_>, recovery: TokenSet) {
    if p.at(IDENT) {
        let m = p.start();
        p.bump(IDENT);
        m.complete(p, NAME_REF);
    } else {
        p.err_recover("expected a name", recovery);
    }
}

fn name_ref(p: &mut Parser<'_>) {
    name_ref_r(p, TokenSet::EMPTY);
}

fn error_block(p: &mut Parser<'_>, message: &str) {
    assert!(p.at(LEFT_CURLY));
    let m = p.start();
    p.error(message);
    p.bump(LEFT_CURLY);
    while !p.at(EOF) && !p.at(RIGHT_CURLY) {
        p.bump_any();
    }
    p.eat(RIGHT_CURLY);
    m.complete(p, ERROR);
}

pub(super) fn qual_ident(p: &mut Parser<'_>) {}

fn member_list(
    p: &mut Parser,
    bra: SyntaxKind,
    ket: SyntaxKind,
    member: impl Fn(&mut Parser),
    delim: SyntaxKind,
    list_kind: SyntaxKind,
    expected_error_msg: &'static str,
) {
    assert!(p.at(bra));
    let m = p.start();
    p.bump(bra);

    while !p.at(ket) && !p.at(EOF) {
        if p.at(bra) {
            error_block(p, expected_error_msg);
            continue;
        }

        member(p);

        if !p.at(ket) {
            if p.at(EOL) {
                p.bump(EOL);
            } else {
                p.expect(delim);
            }
        }
    }

    m.complete(p, list_kind);
}

fn expr_opt(p: &mut Parser, prefix: SyntaxKind, rule: SyntaxKind) {
    if p.at(prefix) {
        let m = p.start();
        p.bump(prefix);
        expr::expr(p);
        m.complete(p, rule);
    }
}

fn size(p: &mut Parser) {
    assert!(p.at(LEFT_SQUARE));
    let m = p.start();
    p.bump(LEFT_SQUARE);
    expr::expr(p);
    m.complete(p, SIZE);
    p.expect(RIGHT_SQUARE);
}

fn size_opt(p: &mut Parser) {
    if p.at(LEFT_SQUARE) {
        size(p);
    }
}

fn formal_param_list(p: &mut Parser) {
    if p.at(LEFT_PAREN) {
        member_list(
            p,
            LEFT_PAREN,
            RIGHT_PAREN,
            formal_param,
            COMMA,
            FORMAL_PARAM_LIST,
            "expected formal parameter",
        )
    }
}

fn formal_param(p: &mut Parser) {
    let m = p.start();
    p.eat(REF_KW);
    if p.at(IDENT) {
        name(p);

        if p.eat(COLON) {
            types::type_name(p);
        } else {
            p.error("expected `:`");
        }

        m.complete(p, DEF_ENUM_CONSTANT);
    } else {
        m.abandon(p);
        p.err_and_bump("expected formal parameter");
    }
}

fn format_(p: &mut Parser) {
    let m = p.start();
    expr::lit_string(p);
    m.complete(p, FORMAT);
}

fn format_opt(p: &mut Parser) {
    if p.eat(FORMAT_KW) {
        format_(p);
    }
}

fn spec_include(p: &mut Parser) {}
