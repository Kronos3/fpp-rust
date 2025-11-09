use fpp_core::SourceFile;
use pretty_assertions::assert_eq;
use std::path::PathBuf;
use std::{env, fs};

pub(crate) fn run_test(file_path: &str) {
    // Compute the path to the FPP input and .ref.txt output
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/test");

    let mut fpp_file = path.clone();
    fpp_file.push(file_path);
    fpp_file.set_extension("fpp");

    let mut ref_file = path.clone();
    ref_file.push(file_path);
    ref_file.set_extension("ref.txt");

    // Set up the compiler context to capture diagnostic messages into a buffer
    let mut diagnostics_str = vec![];
    let mut ctx =
        fpp_core::CompilerContext::new(fpp_errors::WriteEmitter::new(&mut diagnostics_str));

    // Parse the input and run the semantic checker on the AST
    fpp_core::run(&mut ctx, || {
        let source_file_path = fpp_file.to_str().unwrap();
        let src = match SourceFile::open(source_file_path) {
            Ok(src) => src,
            Err(err) => panic!("failed to open {}: {}", source_file_path, err.to_string()),
        };

        let mut ast = fpp_parser::parse(src, |p| p.trans_unit(), None);
        let mut a = crate::Analysis::new();
        let _ = crate::passes::check_semantics(&mut a, &mut ast);
    })
    .expect("compiler_error");

    let output = String::from_utf8(diagnostics_str)
        .expect("failed to convert error message to string")
        .replace(path.to_str().unwrap(), "[ local path prefix ]");

    // Validate the diagnostic messages against the reference file
    match env::var("FPP_UPDATE_REF") {
        Ok(_) => {
            // Update the ref file
            fs::write(ref_file, output).expect("failed to write ref.txt")
        }
        Err(_) => {
            // Read and compare against the ref file
            let ref_txt = fs::read_to_string(ref_file).expect("failed to read ref.txt");
            assert_eq!(ref_txt, output)
        }
    }
}
