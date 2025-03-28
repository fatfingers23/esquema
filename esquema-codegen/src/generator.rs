// Forked from atrium-codegen
// https://github.com/sugyan/atrium/blob/main/lexicon/atrium-codegen/src/generator.rs

use crate::fs::find_dirs;
use crate::schema::find_ref_unions;
use crate::token_stream::{
    collection, enum_common, impl_into_record, lexicon_module, modules, ref_unions, user_type,
};
use atrium_lex::LexiconDoc;
use atrium_lex::lexicon::LexUserType;
use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use std::error::Error;
use std::fs::{File, create_dir_all, read_dir};
use std::io::Write;
use std::path::{Path, PathBuf};

const HEADER: &str = "// @generated - This file is generated by esquema-codegen (forked from atrium-codegen). DO NOT EDIT.";

pub(crate) fn generate_schemas(
    schema: &LexiconDoc,
    outdir: &Path,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut results = Vec::new();
    let mut paths = schema.id.split('.').collect::<Vec<_>>();
    if let Some(basename) = paths.pop() {
        let mut tokens = Vec::new();
        let mut names = Vec::new();
        for (name, def) in &schema.defs {
            // NSID (for XRPC Query, Procedure, Subscription)
            if matches!(
                def,
                LexUserType::XrpcQuery(_)
                    | LexUserType::XrpcProcedure(_)
                    | LexUserType::XrpcSubscription(_)
            ) {
                let nsid = schema.id.clone();
                tokens.push(quote! {
                    pub const NSID: &str = #nsid;
                });
            }
            // main def
            if name == "main" {
                tokens.push(user_type(def, &schema.id, basename, true)?);
            } else {
                names.push(name);
            }
        }
        // other defs
        for &name in names.iter().sorted() {
            tokens.push(user_type(&schema.defs[name], &schema.id, name, false)?);
        }
        // ref unions
        tokens.push(ref_unions(&schema.id, &find_ref_unions(&schema.defs))?);

        let documentation = {
            let doc = format!("Definitions for the `{}` namespace.", schema.id);
            let description = if let Some(description) = &schema.description {
                quote!(#![doc = #description])
            } else {
                quote!()
            };
            quote! {
                #![doc = #doc]
                #description
            }
        };
        let content = quote! {
            #documentation
            #(#tokens)*
        };
        let dir = outdir.join(paths.join("/"));
        create_dir_all(&dir)?;
        let mut filename = PathBuf::from(basename.to_snake_case());
        filename.set_extension("rs");
        let path = dir.join(filename);
        write_to_file(File::create(&path)?, content)?;
        results.push(path);
    }
    Ok(results)
}

pub(crate) fn generate_records(
    outdir: &Path,
    schemas: &[LexiconDoc],
    namespaces: &[(String, Option<&str>)],
) -> Result<PathBuf, Box<dyn Error>> {
    let records = schemas
        .iter()
        .filter_map(|schema| {
            if let Some(LexUserType::Record(_)) = schema.defs.get("main") {
                Some(schema.id.clone())
            } else {
                None
            }
        })
        .sorted()
        .collect_vec();
    let known_record = enum_common(&records, "KnownRecord", None, namespaces)?;
    let impl_into = impl_into_record(&records, namespaces)?;
    let content = quote! {
        #![doc = "A collection of known record types."]
        #known_record
        #impl_into
    };
    let path = outdir.join("record.rs");
    write_to_file(File::create(&path)?, content)?;
    Ok(path)
}

pub(crate) fn generate_lexicons_mod(
    outdir: &Path,
    namespaces: &[(String, Option<&str>)],
) -> Result<PathBuf, Box<dyn Error>> {
    let module = lexicon_module(namespaces)?;
    let path = outdir.join("mod.rs");
    write_to_file(File::create(&path)?, module)?;

    Ok(path)
}

pub(crate) fn generate_modules(
    outdir: &Path,
    schemas: &[LexiconDoc],
    namespaces: &[(String, Option<&str>)],
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths = find_dirs(outdir)?;
    paths.reverse();
    paths.retain(|p| {
        p.as_ref() != outdir
            && p.as_ref()
                .strip_prefix(outdir)
                .map_or(true, |p| !p.starts_with("agent") && !p.starts_with("types"))
    });
    let mut files = Vec::with_capacity(paths.len());
    // generate ".rs" files names
    for path in &paths {
        let mut p = path.as_ref().to_path_buf();
        p.set_extension("rs");
        files.push(p);
    }
    // write "mod" statements
    for (path, filepath) in paths.iter().zip(&files) {
        let names = read_dir(path)?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .filter_map(|entry| {
                entry
                    .path()
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
            })
            .sorted()
            .collect_vec();
        let relative = path.as_ref().strip_prefix(outdir)?;
        let modules = modules(
            &names,
            &relative
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect_vec(),
            namespaces,
        )?;
        let (documentation, collections) = {
            let ns = relative.to_string_lossy().replace(['/', '\\'], ".");
            let doc = format!("Definitions for the `{}` namespace.", ns);
            let collections = names
                .iter()
                .filter_map(|name| {
                    let nsid = format!("{}.{}", ns, name);
                    schemas
                        .iter()
                        .find(|schema| {
                            schema
                                .defs
                                .get("main")
                                .map(|def| {
                                    schema.id == nsid && matches!(def, LexUserType::Record(_))
                                })
                                .unwrap_or(false)
                        })
                        .map(|_| collection(name, &nsid))
                })
                .collect_vec();
            (quote!(#![doc = #doc]), collections)
        };
        let content = quote! {
            #documentation
            #modules
            #(#collections)*
        };
        write_to_file(File::create(filepath)?, content)?;
    }
    Ok(files)
}

fn write_to_file(mut file: impl Write, content: TokenStream) -> Result<(), Box<dyn Error>> {
    let parsed = syn::parse_file(&content.to_string())?;
    writeln!(file, "{HEADER}")?;
    write!(file, "{}", prettyplease::unparse(&parsed))?;
    Ok(())
}
