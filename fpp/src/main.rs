use std::io::Read;
use std::process::exit;

fn compiler_main() -> String {
    let mut stdin = String::new();
    std::io::stdin()
        .read_to_string(&mut stdin)
        .expect("Failed to read input stream");

    let src = fpp_core::SourceFile::from(stdin.as_str());
    let mut ast = fpp_parser::parse(src, |p| p.trans_unit(), None);

    let mut a = fpp_analysis::Analysis::new();

    let _ = fpp_analysis::passes::check_semantics(&mut a, &mut ast);

    format!("{:#?}", ast)
}

fn main() {
    let mut ctx = fpp_core::CompilerContext::new(fpp_errors::ConsoleEmitter::color());
    let out = fpp_core::run(&mut ctx, compiler_main).expect("Failed to run compiler");

    if ctx.has_errors() {
        exit(1)
    }

    println!("{}", out);
}
