use std::io::Read;

fn compiler_main() -> String {
    let mut stdin = String::new();
    std::io::stdin()
        .read_to_string(&mut stdin)
        .expect("Failed to read input stream");

    let src = fpp_core::SourceFile::from(stdin.as_str());

    let ast = fpp_parser::parse(src, |p| p.module_members());
    format!("{:#?}", ast)
}

fn main() {
    let mut ctx =
        fpp_core::CompilerContext::new(fpp_errors::ConsoleEmitter::color());
    let out = fpp_core::run(&mut ctx, compiler_main).expect("Failed to run compiler");

    println!("{}", out)
}
