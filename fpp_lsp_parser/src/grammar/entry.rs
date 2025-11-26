use super::*;
use crate::grammar::module::module_member;

pub(crate) fn module_entry(p: &mut Parser) {
    let m = p.start();
    while !p.at(EOF) {
        if p.at(LEFT_CURLY) {
            error_block(p, "expected module member");
            continue;
        }

        while p.at(EOL) || p.at(SEMI) {
            p.bump_any();
        }

        if p.at(EOF) {
            break;
        }

        module_member(p);
        match p.current() {
            SEMI | EOL | EOF => {}
            _ => {
                p.err_recover("expected `;`", MEMBER_RECOVERY_SET);
            }
        }
    }

    m.complete(p, ROOT);
}

pub(crate) fn component_entry(p: &mut Parser) {
    // let m = p.start();
    todo!();
    // m.complete(p, ROOT);
}

pub(crate) fn topology_entry(p: &mut Parser) {
    let m = p.start();

    m.complete(p, ROOT);
}

pub(crate) fn tlm_packet_entry(p: &mut Parser) {
    let m = p.start();

    m.complete(p, ROOT);
}

pub(crate) fn tlm_packet_set_entry(p: &mut Parser) {
    let m = p.start();

    m.complete(p, ROOT);
}
