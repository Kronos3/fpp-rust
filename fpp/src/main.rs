fn compiler_main() -> Result<(), fpp_core::Error> {
    let src = fpp_core::SourceFile::from("");

    match fpp_parser::parse(src, |p| p.module_members()) {
        Ok(ast) => {
            println!("{:#?}", ast);
            Ok(())
        }
        Err(err) => Err(fpp_core::Error::from(format!("{:?}", err))),
    }
}

fn main() {
    let mut ctx = fpp_core::CompilerContext::new();
    fpp_core::run(&mut ctx, || match compiler_main() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
        }
    })
        .expect("Failed to run compiler");
}
