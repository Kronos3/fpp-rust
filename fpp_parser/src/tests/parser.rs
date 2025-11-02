use crate::parse;
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
    let ast: String = fpp_core::run(&mut ctx, || {
        let source_file_path = fpp_file.to_str().unwrap();
        let src = match SourceFile::open(source_file_path) {
            Ok(src) => src,
            Err(err) => panic!("failed to open {}: {}", source_file_path, err.to_string()),
        };

        // Parse the source
        format!("{:#?}", parse(src, |p| p.module_members()))
    })
    .expect("compiler_error");

    let output = if diagnostics_str.is_empty() {
        ast
    } else {
        String::from_utf8(diagnostics_str).expect("failed to convert error message to string")
    }
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

macro_rules! parser_tests {
    ($($name:ident: $path:expr,)*) => {
    $(
        #[test]
        fn $name () {
            run_test($path)
        }
    )*
    };
}

parser_tests!(
    comments: "comments",
    cycle_1: "cycle-1",
    cycle_2: "cycle-2",
    cycle_3: "cycle-3",
    embedded_tab: "embedded-tab",
    empty: "empty",
    escaped_strings: "escaped-strings",
    illegal_character: "illegal-character",
    include_component: "include-component",
    include_constant_1: "include-constant-1",
    include_missing_file: "include-missing-file",
    include_module: "include-module",
    include_parse_error: "include-parse-error",
    include_subdir: "include-subdir",
    include_topology: "include-topology",
    parse_error: "parse-error",
    state_machine: "state-machine",
    syntax: "syntax",
    syntax_kwd_names: "syntax-kwd-names",
    topology_ports: "topology-ports",
);
