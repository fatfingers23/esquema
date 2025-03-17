use std::path::PathBuf;

use esquema_codegen::genapi;

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
    let lex_dir = PathBuf::from("./lexicons");
    let output = PathBuf::from(out_dir);
    let _ = genapi(&lex_dir, &output).unwrap();
}
