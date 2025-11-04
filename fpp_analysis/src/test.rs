use fpp_core::SourceFile;
use std::path::PathBuf;
use std::{env, fs};

fn run_test(file_path: &str) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/tests");

    let mut fpp_file = path.clone();
    fpp_file.push(file_path);
    fpp_file.set_extension("fpp");

    let mut ref_file = path.clone();
    ref_file.push(file_path);
    ref_file.set_extension("ref.txt");

    let mut diagnostics_str = vec![];
    let mut ctx =
        fpp_core::CompilerContext::new(fpp_errors::WriteEmitter::new(&mut diagnostics_str));
    fpp_core::run(&mut ctx, || {
        let source_file_path = fpp_file.to_str().unwrap();
        let src = match SourceFile::open(source_file_path) {
            Ok(src) => src,
            Err(err) => panic!("failed to open {}: {}", source_file_path, err.to_string()),
        };

        let ast = fpp_parser::parse(src, |p| p.module_members());

        let mut a = crate::Analysis::new();

        crate::passes::check_semantics(&mut a, &ast);
    })
    .expect("compiler_error");

    let output = String::from_utf8(diagnostics_str)
        .expect("failed to convert error message to string")
        .replace(path.to_str().unwrap(), "[ local path prefix ]");

    match env::var("FPP_UPDATE_REF") {
        Ok(_) => {
            // Update the ref file
            fs::write(ref_file, output).expect("failed to write ref.txt")
        }
        Err(_) => {
            // Read and compare against the ref file
            let ref_txt = fs::read_to_string(ref_file).expect("failed to read ref.txt");
            assert_eq!(output, ref_txt)
        }
    }
}

macro_rules! check_tests {
    ($($name:ident: $path:expr,)*) => {
    $(
        #[test]
        fn $name () {
            run_test($path)
        }
    )*
    };
}

check_tests!(
    duplicate_symbols: "duplicate-symbols",
);
