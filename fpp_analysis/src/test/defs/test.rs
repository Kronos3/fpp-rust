use crate::test::lib::run_test;


#[test]
fn duplicate_symbols_single() {
    run_test("defs/duplicate-symbols-single")
}

#[test]
fn duplicate_symbols_multi() {
    run_test("defs/duplicate-symbols-multi")
}

#[test]
fn ok() {
    run_test("defs/ok")
}
