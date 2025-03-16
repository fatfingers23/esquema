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
use itertools::Itertools;
use serde_json::from_reader;
use std::error::Error;
use std::fs::File;
use std::option::Option;
use std::path::{Path, PathBuf};

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
    let mut results = Vec::new();
    //HACK had to change to String instead of &str, but keeping as tuple for now to match atrium-codegen
    let mut namespaces: Vec<(String, Option<&str>)> = Vec::new();

    //HACK not sure if I need that clone
    for doc in schemas.clone() {
        results.extend(generate_schemas(&doc.clone(), &outdir)?);
        let namespace = doc
            .id
            .rsplit_once('.')
            .map_or(doc.id.clone(), |(prefix, _)| prefix.to_string());
        namespaces.push((namespace, None));
    }

    results.push(generate_records(&outdir, &schemas, &namespaces)?);
    results.push(generate_lexicons_mod(&outdir, &namespaces)?);
    results.extend(generate_modules(&outdir, &schemas, &namespaces)?);

    Ok(results)
}

fn generate(outdir: &Path, schemas: &[&LexiconDoc]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut results = Vec::new();
    for &schema in schemas {
        results.extend(generate_schemas(schema, outdir)?);
    }
    Ok(results)
}
