use std::cell::RefCell;
use std::io::Read;
use std::process::exit;
use std::rc::Rc;

fn compiler_main() -> String {
    let mut stdin = String::new();
    std::io::stdin()
        .read_to_string(&mut stdin)
        .expect("Failed to read input stream");

    let src = fpp_core::SourceFile::new("<stdin>", stdin);
    let mut ast = fpp_parser::parse(src, |p| p.trans_unit(), None);

    let mut a = fpp_analysis::Analysis::new();

    let _ = fpp_analysis::passes::check_semantics(&mut a, Box::new(fpp_fs::FsReader {}), &mut ast);

    format!("{:#?}", ast)
}

fn main() {
    let diagnostics = Rc::new(RefCell::new(fpp_errors::ConsoleEmitter::color()));
    let mut ctx = fpp_core::CompilerContext::new(diagnostics.clone());
    let out = fpp_core::run(&mut ctx, compiler_main).expect("Failed to run compiler");

    if diagnostics.borrow().has_errors() {
        exit(1)
    }

    println!("{}", out);
}
