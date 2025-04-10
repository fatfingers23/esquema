// Forked from atrium-codegen
// https://github.com/sugyan/atrium/blob/main/lexicon/atrium-codegen/src/lib.rs

mod fs;
mod generator;
mod schema;
mod token_stream;

use crate::generator::{
    generate_lexicons_mod, generate_modules, generate_records, generate_schemas,
};
use atrium_lex::LexiconDoc;
use serde_json::from_reader;
use std::error::Error;
use std::fs::File;
use std::option::Option;
use std::path::Path;

pub fn genapi(
    lexdir: impl AsRef<Path>,
    outdir: impl AsRef<Path>,
) -> Result<Vec<impl AsRef<Path>>, Box<dyn Error>> {
    let lexdir = lexdir.as_ref().canonicalize()?;
    let outdir = outdir.as_ref().canonicalize()?;
    let paths = fs::find_schemas(&lexdir)?;
    let mut schemas = Vec::with_capacity(paths.len());
    for path in &paths {
        schemas.push(from_reader::<_, LexiconDoc>(File::open(path)?)?);
    }
    gen_from_lexicon_docs(schemas, outdir)
}

pub fn gen_from_lexicon_docs(
    schemas: Vec<LexiconDoc>,
    outdir: impl AsRef<Path>,
) -> Result<Vec<impl AsRef<Path>>, Box<dyn Error>> {
    let outdir = outdir.as_ref().canonicalize()?;
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

    results.push(generate_records(&outdir, &schemas, &namespaces)?);
    results.push(generate_lexicons_mod(&outdir, &namespaces)?);
    results.extend(generate_modules(&outdir, &schemas, &namespaces)?);

    Ok(results)
}
