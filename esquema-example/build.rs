use std::path::PathBuf;

use esquema_codegen::genapi;

fn main() {
    //TODO find a way to only tun this if the lexicons have changed?
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
    let lex_dir = PathBuf::from("./lexicons");
    let output = PathBuf::from(out_dir);
    let _ = genapi(&lex_dir, &output).unwrap();
}
