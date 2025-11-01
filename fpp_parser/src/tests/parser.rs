use crate::{parse, ParseError};
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

    let mut ctx = fpp_core::CompilerContext::new();
    let res = fpp_core::run(&mut ctx, || {
        let src = match SourceFile::open(fpp_file.to_str().unwrap()) {
            Ok(src) => src,
            Err(err) => return Err(ParseError::FileOpen { error: err }),
        };

        parse(src, |p| p.module_members())
    })
    .expect("compiler_error");

    let output = fpp_core::run(&mut ctx, || match res {
        Ok(ast) => format!("{:#?}", ast),
        Err(err) => format!("{:#?}", err),
    })
    .expect("compiler error")
    .replace(path.to_str().unwrap(), "[ local path prefix ]");

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
    include_constant_1: "include_constant_1",
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
