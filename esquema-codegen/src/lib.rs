// Forked from atrium-codegen
// https://github.com/sugyan/atrium/blob/main/lexicon/atrium-codegen/src/lib.rs

mod fs;
mod generator;
mod schema;
mod token_stream;

use crate::generator::{
    generate_client, generate_lexicons_mod_or_lib, generate_modules, generate_records,
    generate_schemas,
};
use atrium_lex::LexiconDoc;
use serde_json::from_reader;
use std::error::Error;
use std::fs::File;
use std::option::Option;
use std::path::{Path, PathBuf};

fn canonicalize_and_validate(
    path: impl AsRef<Path>,
    error_msg: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    match path.as_ref().canonicalize() {
        Ok(dir) => {
            if !dir.exists() {
                return Err(format!("{error_msg}: {:?}", dir.as_path()).into());
            }
            Ok(dir)
        }
        Err(_) => Err(format!("{error_msg}: {:?}", path.as_ref().as_os_str()).into()),
    }
}

pub fn genapi(
    lexdir: impl AsRef<Path>,
    outdir: impl AsRef<Path>,
    module_name: &Option<String>,
) -> Result<Vec<impl AsRef<Path>>, Box<dyn Error>> {
    let lexdir = canonicalize_and_validate(lexdir, "Lexicon directory does not exist")?;
    let outdir = canonicalize_and_validate(outdir, "Output directory does not exist")?;

    let paths = fs::find_schemas(&lexdir)?;
    let mut schemas = Vec::with_capacity(paths.len());
    for path in &paths {
        schemas.push(from_reader::<_, LexiconDoc>(File::open(path)?)?);
    }
    gen_from_lexicon_docs(schemas, outdir, &module_name)
}

pub fn gen_from_lexicon_docs(
    schemas: Vec<LexiconDoc>,
    outdir: impl AsRef<Path>,
    module_name: &Option<String>,
) -> Result<Vec<impl AsRef<Path>>, Box<dyn Error>> {
    let mut outdir = canonicalize_and_validate(outdir, "Output directory does not exist")?;
    if let Some(module_name) = module_name {
        outdir.push(module_name);
    }
    if !outdir.exists() {
        return Err(format!("Output directory does not exist: {:?}", outdir).into());
    }
    let mut results = Vec::new();
    //HACK had to change to String instead of &str, but keeping as tuple for now to match atrium-codegen
    let mut namespaces: Vec<(String, Option<&str>)> = Vec::new();

    //HACK not sure if I need that clone
    for doc in schemas.clone() {
        results.extend(generate_schemas(&doc.clone(), &outdir)?);
        //TODO do proper error handling
        let parts: Vec<&str> = doc.id.split('.').collect();
        let namespace = format!("{}.{}", parts[0], parts[1]);
        //TODO prob just move to [String] for name spaces since im hard coding in None
        if namespaces.iter().any(|x| x.0 == namespace) {
            continue;
        }
        namespaces.push((namespace, None));
    }

    results.push(generate_records(
        &outdir,
        &schemas,
        &namespaces,
        &module_name,
    )?);
    results.push(generate_client(&outdir, &schemas, &namespaces)?);

    results.push(generate_lexicons_mod_or_lib(
        &outdir,
        &namespaces,
        module_name.is_none(),
    )?);
    results.extend(generate_modules(&outdir, &schemas, &namespaces)?);

    Ok(results)
}
