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

#[test]
fn comments() {
    run_test("comments")
}

#[test]
fn cycle_1() {
    run_test("cycle-1")
}

#[test]
fn cycle_2() {
    run_test("cycle-2")
}

#[test]
fn cycle_3() {
    run_test("cycle-3")
}

#[test]
fn embedded_tab() {
    run_test("embedded-tab")
}

#[test]
fn empty() {
    run_test("empty")
}

#[test]
fn escaped_strings() {
    run_test("escaped-strings")
}

#[test]
fn illegal_character() {
    run_test("illegal-character")
}

#[test]
fn include_component() {
    run_test("include-component")
}

#[test]
fn include_constant_1() {
    run_test("include-constant-1")
}

#[test]
fn include_missing_file() {
    run_test("include-missing-file")
}

#[test]
fn include_module() {
    run_test("include-module")
}

#[test]
fn include_parse_error() {
    run_test("include-parse-error")
}

#[test]
fn include_subdir() {
    run_test("include-subdir")
}

#[test]
fn include_topology() {
    run_test("include-topology")
}

#[test]
fn parse_error() {
    run_test("parse-error")
}

#[test]
fn state_machine() {
    run_test("state-machine")
}

#[test]
fn syntax() {
    run_test("syntax")
}

#[test]
fn syntax_kwd_names() {
    run_test("syntax-kwd-names")
}

#[test]
fn topology_ports() {
    run_test("topology-ports")
}
