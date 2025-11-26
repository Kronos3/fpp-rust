use super::*;

pub(super) fn def_component(p: &mut Parser) {
    let m = p.start();
    match p.current() {
        ACTIVE_KW | PASSIVE_KW | QUEUED_KW => {
            p.bump_any();
        }
        _ => unreachable!(),
    }

    p.expect(COMPONENT_KW);
    name_r(p, MEMBER_RECOVERY_SET);

    if p.at(LEFT_CURLY) {
        member_list(
            p,
            LEFT_CURLY,
            RIGHT_CURLY,
            component_member,
            SEMI,
            COMPONENT_MEMBER_LIST,
            "expected component member",
        );
    } else {
        p.error("expected `{`");
    }

    m.complete(p, DEF_COMPONENT);
}

pub(super) fn component_member(p: &mut Parser) {
    match p.current() {
        TYPE_KW => types::type_alias_or_abstract(p),
        ARRAY_KW => types::def_array(p),
        CONSTANT_KW => module::def_constant(p),
        ENUM_KW => types::def_enum(p),
        STATE_KW if p.nth_at(2, INSTANCE_KW) => spec_state_machine_instance(p),
        STATE_KW => state_machine::def_state_machine(p),
        STRUCT_KW => types::def_struct(p),
        ASYNC_KW | GUARD_KW | SYNC_KW if p.nth_at(1, COMMAND_KW) => {
            spec_command(p);
        }
        ASYNC_KW | GUARD_KW | SYNC_KW | OUTPUT_KW | COMMAND_KW | TEXT_KW | TIME_KW => {
            spec_port_instance(p);
        }
        PRODUCT_KW if p.nth_at(1, CONTAINER_KW) => spec_container(p),
        PRODUCT_KW if p.nth_at(1, RECORD_KW) => spec_record(p),
        PRODUCT_KW => spec_port_instance(p),
        EVENT_KW if p.nth_at(1, PORT_KW) => spec_port_instance(p),
        EVENT_KW => spec_event(p),
        INCLUDE_KW => spec_include(p),
        MATCH_KW => spec_port_matching(p),
        PARAM_KW if p.nth_at(1, PORT_KW) => spec_port_instance(p),
        EXTERNAL_KW | PARAM_KW => spec_param(p),
        TELEMETRY_KW if p.nth_at(1, PORT_KW) => spec_port_instance(p),
        TELEMETRY_KW => spec_telemetry(p),
        IMPORT_KW => module::spec_import_interface(p),
        _ => {
            p.err_recover("component member expected", MEMBER_RECOVERY_SET);
        }
    }
}

fn spec_state_machine_instance(p: &mut Parser) {
    assert!(p.at(STATE_KW));
    let m = p.start();
    p.bump(STATE_KW);
    p.expect(MACHINE_KW);
    p.expect(INSTANCE_KW);

    name_ref(p);
    p.expect(COLON);
    qual_ident(p);

    expr_opt(p, PRIORITY_KW, PRIORITY);
    queue_full_opt(p);

    m.complete(p, SPEC_STATE_MACHINE_INSTANCE);
}

fn spec_command(p: &mut Parser) {
    let m = p.start();
    match p.current() {
        ASYNC_KW | GUARD_KW | SYNC_KW => p.bump_any(),
        _ => unreachable!(),
    }

    p.bump(COMMAND_KW);
    name_r(p, MEMBER_RECOVERY_SET);
    formal_param_list(p);

    expr_opt(p, OPCODE_KW, OPCODE);
    expr_opt(p, PRIORITY_KW, PRIORITY);
    queue_full_opt(p);

    m.complete(p, SPEC_COMMAND);
}

fn queue_full_opt(p: &mut Parser) {
    match p.current() {
        ASSERT_KW | BLOCK_KW | DROP_KW | HOOK_KW => {
            let m = p.start();
            p.bump_any();
            m.complete(p, QUEUE_FULL);
        }
        _ => {}
    }
}

pub(super) fn spec_port_instance(p: &mut Parser) {
    match p.current() {
        ASYNC_KW | GUARD_KW | SYNC_KW if p.nth_at(1, INPUT_KW) => spec_port_instance_general(p),
        OUTPUT_KW => spec_port_instance_general(p),
        _ => spec_port_instance_special(p),
    }
}

fn spec_port_instance_general(p: &mut Parser) {
    let m = p.start();

    match p.current() {
        ASYNC_KW | GUARD_KW | SYNC_KW => {
            p.bump_any();
            p.bump(INPUT_KW);
        }
        OUTPUT_KW => p.bump(OUTPUT_KW),
        _ => unreachable!(),
    }

    p.expect(PORT_KW);
    name(p);
    p.expect(COLON);
    index_or_size_opt(p);

    if p.at(SERIAL_KW) {
        p.bump(SERIAL_KW);
    } else {
        qual_ident(p);
    }

    expr_opt(p, PRIORITY_KW, PRIORITY);
    queue_full_opt(p);
    m.complete(p, SPEC_PORT_INSTANCE_GENERAL);
}

fn spec_port_instance_special(p: &mut Parser) {
    let m = p.start();

    match p.current() {
        ASYNC_KW | SYNC_KW | GUARD_KW => {
            p.bump_any();
        }
        _ => {}
    }

    if p.eat(COMMAND_KW) {
        match p.current() {
            RECV_KW | REG_KW | RESP_KW => {
                p.bump_any();
            }
            _ => {
                m.abandon(p);
                p.err_recover("expected `get` or `set`", MEMBER_RECOVERY_SET);
                return;
            }
        }
    } else if p.eat(PARAM_KW) {
        match p.current() {
            GET_KW | SET_KW => {
                p.bump_any();
            }
            _ => {
                m.abandon(p);
                p.err_recover("expected `get` or `set`", MEMBER_RECOVERY_SET);
                return;
            }
        }
    } else if p.eat(PRODUCT_KW) {
        match p.current() {
            GET_KW | RECV_KW | REQUEST_KW | SEND_KW => {
                p.bump_any();
            }
            _ => {
                m.abandon(p);
                p.err_recover(
                    "expected `get`, `recv`, `request`, or `send`",
                    MEMBER_RECOVERY_SET,
                );
                return;
            }
        }
    } else if p.eat(TEXT_KW) {
        p.expect(EVENT_KW);
    } else if p.eat(TIME_KW) {
        p.eat(GET_KW);
    } else {
        match p.current() {
            EVENT_KW | TELEMETRY_KW => {
                p.bump_any();
            }
            _ => {
                m.abandon(p);
                p.err_recover("expected special port instance", MEMBER_RECOVERY_SET);
                return;
            }
        }
    }

    p.expect(PORT_KW);
    name(p);
    expr_opt(p, PRIORITY_KW, PRIORITY);
    queue_full_opt(p);
    m.complete(p, SPEC_PORT_INSTANCE_SPECIAL);
}

fn spec_container(p: &mut Parser) {
    assert!(p.at(PRODUCT_KW));
    let m = p.start();
    p.bump(PRODUCT_KW);
    p.bump(CONTAINER_KW);

    name(p);
    expr_opt(p, ID_KW, ID);
    if p.eat(DEFAULT_KW) {
        p.expect(PRIORITY_KW);
        let m = p.start();
        expr::expr(p);
        m.complete(p, DEFAULT_PRIORITY);
    }

    m.complete(p, SPEC_CONTAINER);
}

fn spec_record(p: &mut Parser) {
    assert!(p.at(PRODUCT_KW));
    let m = p.start();
    p.bump(PRODUCT_KW);
    p.bump(CONTAINER_KW);

    name(p);
    p.expect(COLON);
    types::type_name(p);

    p.eat(ARRAY_KW);
    expr_opt(p, ID_KW, ID);

    m.complete(p, SPEC_RECORD);
}

fn spec_event(p: &mut Parser) {
    assert!(p.at(EVENT_KW));
    let m = p.start();
    p.bump(EVENT_KW);
    name(p);
    formal_param_list(p);

    match p.current() {
        ACTIVITY_KW | WARNING_KW => {
            p.bump_any();
            match p.current() {
                HIGH_KW | LOW_KW => {
                    p.bump_any();
                }
                _ => p.err_and_bump("expected `high` or `low`"),
            }
        }
        COMMAND_KW | DIAGNOSTIC_KW | FATAL_KW => p.bump_any(),
        _ => p.err_and_bump("severity level expected"),
    }

    expr_opt(p, ID_KW, ID);
    format_(p);

    if p.at(THROTTLE_KW) {
        event_throttle(p);
    }

    m.complete(p, SPEC_EVENT);
}

fn event_throttle(p: &mut Parser) {
    assert!(p.at(THROTTLE_KW));
    let m = p.start();
    p.bump(THROTTLE_KW);
    expr::expr(p);
    expr_opt(p, EVERY_KW, EVERY);
    m.complete(p, EVENT_THROTTLE);
}

fn spec_port_matching(p: &mut Parser) {
    assert!(p.at(MATCH_KW));
    let m = p.start();
    p.bump(MATCH_KW);
    name_ref(p);
    p.expect(WITH_KW);
    name_ref(p);

    m.complete(p, MATCH_KW);
}

fn spec_param(p: &mut Parser) {
    let m = p.start();
    p.eat(EXTERNAL_KW);
    p.bump(PARAM_KW);
    name_r(p, MEMBER_RECOVERY_SET);
    p.expect(COLON);
    types::type_name(p);
    expr_opt(p, DEFAULT_KW, DEFAULT);
    expr_opt(p, ID_KW, ID);

    if p.eat(SET_KW) {
        p.expect(OPCODE_KW);
        let m = p.start();
        expr::expr(p);
        m.complete(p, SET_OPCODE);
    }

    if p.eat(SAVE_KW) {
        p.expect(OPCODE_KW);
        let m = p.start();
        expr::expr(p);
        m.complete(p, SAVE_OPCODE);
    }

    m.complete(p, PARAM_KW);
}

fn spec_telemetry(p: &mut Parser) {
    assert!(p.at(TELEMETRY_KW));
    let m = p.start();
    p.bump(TELEMETRY_KW);
    name_r(p, MEMBER_RECOVERY_SET);
    p.expect(COLON);
    types::type_name(p);
    expr_opt(p, ID_KW, ID);

    if p.eat(UPDATE_KW) {
        match p.current() {
            ALWAYS_KW | ON_KW => {
                p.bump_any();
            }
            _ => {}
        }
    }

    if p.eat(LOW_KW) {
        limit_sequence(p);
    }

    if p.eat(HIGH_KW) {
        limit_sequence(p);
    }

    format_opt(p);
    m.complete(p, SPEC_TELEMETRY);
}

fn limit_sequence(p: &mut Parser) {
    if p.at(LEFT_CURLY) {
        member_list(
            p,
            LEFT_CURLY,
            RIGHT_CURLY,
            limit,
            COMMA,
            LIMIT_SEQUENCE,
            "expected limit",
        );
    } else {
        p.error("expected '{' for limit sequence");
    }
}

fn limit(p: &mut Parser) {
    let m = p.start();
    match p.current() {
        ORANGE_KW | RED_KW | YELLOW_KW => {
            p.bump_any();
        }
        _ => p.err_and_bump("expected telemetry channel limit `orange`, `red` or `yellow`"),
    }

    expr::expr(p);
    m.complete(p, LIMIT);
}
