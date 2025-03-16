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
use std::path::{Path, PathBuf};

pub fn genapi(
    lexdir: impl AsRef<Path>,
    outdir: impl AsRef<Path>,
    namespaces: &[(&str, Option<&str>)],
) -> Result<Vec<impl AsRef<Path>>, Box<dyn Error>> {
    let lexdir = lexdir.as_ref().canonicalize()?;
    let outdir = outdir.as_ref().canonicalize()?;
    let paths = fs::find_schemas(&lexdir)?;
    let mut schemas = Vec::with_capacity(paths.len());
    for path in &paths {
        schemas.push(from_reader::<_, LexiconDoc>(File::open(path)?)?);
    }
    let mut results = Vec::new();
    for &(prefix, _) in namespaces {
        let targets = schemas
            .iter()
            .filter(|schema| schema.id.starts_with(prefix))
            .collect_vec();
        results.extend(generate(&outdir, &targets)?);
    }
    results.push(generate_records(&outdir, &schemas, namespaces)?);
    results.push(generate_lexicons_mod(&outdir, namespaces)?);
    results.extend(generate_modules(&outdir, &schemas, namespaces)?);

    Ok(results)
}

fn generate(outdir: &Path, schemas: &[&LexiconDoc]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut results = Vec::new();
    for &schema in schemas {
        results.extend(generate_schemas(schema, outdir)?);
    }
    Ok(results)
}
