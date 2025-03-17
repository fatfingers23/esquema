// Forked from atrium-codegen
// https://github.com/sugyan/atrium/blob/main/lexicon/atrium-codegen/src/token_stream.rs

use atrium_lex::lexicon::*;
use heck::{ToPascalCase, ToShoutySnakeCase, ToSnakeCase};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::{Path, Result};

pub fn user_type(
    def: &LexUserType,
    schema_id: &str,
    name: &str,
    is_main: bool,
) -> Result<TokenStream> {
    let user_type = match def {
        LexUserType::Record(record) => lex_record(record)?,
        LexUserType::XrpcQuery(query) => lex_query(query)?,
        LexUserType::XrpcProcedure(procedure) => lex_procedure(procedure)?,
        LexUserType::XrpcSubscription(subscription) => lex_subscription(subscription)?,
        LexUserType::Array(array) => lex_array(array, name)?,
        LexUserType::Token(token) => lex_token(token, name, schema_id)?,
        LexUserType::Object(object) => lex_object(object, if is_main { "Main" } else { name })?,
        LexUserType::String(string) => lex_string(string, name)?,
        _ => unimplemented!("{def:?}"),
    };
    Ok(quote! {
        // #[doc = #description]
        #user_type
    })
}

pub fn ref_unions(schema_id: &str, ref_unions: &[(String, LexRefUnion)]) -> Result<TokenStream> {
    let mut enums = Vec::new();
    for (name, ref_union) in ref_unions {
        enums.push(refs_enum(&ref_union.refs, name, Some(schema_id))?);
    }
    Ok(quote!(#(#enums)*))
}

pub fn collection(name: &str, nsid: &str) -> TokenStream {
    let module_name = format_ident!("{name}");
    let collection_name = format_ident!("{}", name.to_pascal_case());
    quote! {
        #[derive(Debug)]
        pub struct #collection_name;
        impl atrium_api::types::Collection for #collection_name {
            const NSID: &'static str = #nsid;
            type Record = #module_name::Record;
        }
    }
}

fn lex_record(record: &LexRecord) -> Result<TokenStream> {
    let LexRecordRecord::Object(object) = &record.record;
    lex_object(object, "Record")
}

fn xrpc_parameters(parameters: &LexXrpcParameters) -> Result<TokenStream> {
    let properties = parameters
        .properties
        .iter()
        .map(|(k, v)| {
            let value = match v {
                LexXrpcParametersProperty::Boolean(boolean) => {
                    LexObjectProperty::Boolean(boolean.clone())
                }
                LexXrpcParametersProperty::Integer(integer) => {
                    LexObjectProperty::Integer(integer.clone())
                }
                LexXrpcParametersProperty::String(string) => {
                    LexObjectProperty::String(string.clone())
                }
                LexXrpcParametersProperty::Unknown(unknown) => {
                    LexObjectProperty::Unknown(unknown.clone())
                }
                LexXrpcParametersProperty::Array(primitive_array) => {
                    LexObjectProperty::Array(LexArray {
                        description: primitive_array.description.clone(),
                        items: match &primitive_array.items {
                            LexPrimitiveArrayItem::Boolean(b) => LexArrayItem::Boolean(b.clone()),
                            LexPrimitiveArrayItem::Integer(i) => LexArrayItem::Integer(i.clone()),
                            LexPrimitiveArrayItem::String(s) => LexArrayItem::String(s.clone()),
                            LexPrimitiveArrayItem::Unknown(u) => LexArrayItem::Unknown(u.clone()),
                        },
                        min_length: primitive_array.min_length,
                        max_length: primitive_array.max_length,
                    })
                }
            };
            (k.clone(), value)
        })
        .collect();
    lex_object(
        &LexObject {
            description: parameters.description.clone(),
            required: parameters.required.clone(),
            nullable: None,
            properties,
        },
        "Parameters",
    )
}

fn xrpc_body(body: &LexXrpcBody, name: &str) -> Result<TokenStream> {
    let description = description(&body.description);
    let schema = if let Some(schema) = &body.schema {
        match schema {
            LexXrpcBodySchema::Ref(r#ref) => {
                let type_name = format_ident!("{}", name.to_pascal_case());
                let (description, ref_type) = ref_type(r#ref)?;
                quote! {
                    #description
                    pub type #type_name = #ref_type;
                }
            }
            LexXrpcBodySchema::Object(object) => lex_object(object, name)?,
            _ => unimplemented!("{schema:?}"),
        }
    } else {
        return Ok(quote!());
    };
    Ok(quote! {
        #description
        #schema
    })
}

fn xrpc_errors(errors: &Option<Vec<LexXrpcError>>) -> Result<TokenStream> {
    let derives = derives()?;
    let errors = errors.as_ref().map_or(Vec::new(), |e| {
        e.iter()
            .map(|error| (error.name.clone(), error.description.clone()))
            .collect()
    });
    let enum_variants: Vec<TokenStream> = errors
        .iter()
        .map(|(name, desc)| {
            let desc = description(desc);
            let name = format_ident!("{}", name.to_pascal_case());
            quote! {
                #desc
                #name(Option<String>)
            }
        })
        .collect();
    let display_arms: Vec<TokenStream> = errors
        .iter()
        .map(|(name, _desc)| {
            let title = name.clone();
            let name = format_ident!("{}", name.to_pascal_case());
            quote! {
                Error::#name(msg) => {
                    write!(_f, #title)?;
                    if let Some(msg) = msg {
                        write!(_f, ": {msg}")?;
                    }
                }
            }
        })
        .collect();
    let body = if display_arms.is_empty() {
        quote!()
    } else {
        quote! {
            match self {
                #(#display_arms)*
            }
        }
    };
    Ok(quote! {
        #derives
        #[serde(tag = "error", content = "message")]
        pub enum Error {
            #(#enum_variants),*
        }
        impl std::fmt::Display for Error {
            fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
                #body
                Ok(())
            }
        }
    })
}

fn lex_query(query: &LexXrpcQuery) -> Result<TokenStream> {
    let params = if let Some(LexXrpcQueryParameter::Params(parameters)) = &query.parameters {
        xrpc_parameters(parameters)?
    } else {
        quote!()
    };
    let outputs = if let Some(body) = &query.output {
        xrpc_body(body, "Output")?
    } else {
        quote!()
    };
    let errors = xrpc_errors(&query.errors)?;
    Ok(quote! {
        #params
        #outputs
        #errors
    })
}

fn lex_procedure(procedure: &LexXrpcProcedure) -> Result<TokenStream> {
    let inputs = if let Some(body) = &procedure.input {
        xrpc_body(body, "Input")?
    } else {
        quote!()
    };
    let outputs = if let Some(body) = &procedure.output {
        xrpc_body(body, "Output")?
    } else {
        quote!()
    };
    let errors = xrpc_errors(&procedure.errors)?;
    Ok(quote! {
        #inputs
        #outputs
        #errors
    })
}

fn lex_subscription(subscription: &LexXrpcSubscription) -> Result<TokenStream> {
    let params =
        if let Some(LexXrpcSubscriptionParameter::Params(parameters)) = &subscription.parameters {
            xrpc_parameters(parameters)?
        } else {
            quote!()
        };
    let errors = xrpc_errors(&subscription.errors)?;
    Ok(quote! {
        #params
        #errors
    })
}

fn lex_array(array: &LexArray, name: &str) -> Result<TokenStream> {
    let (description, array_type) = array_type(array, name, None)?;
    let type_name = format_ident!("{}", name.to_pascal_case());
    Ok(quote! {
        #description
        pub type #type_name = #array_type;
    })
}

fn lex_token(token: &LexToken, name: &str, schema_id: &str) -> Result<TokenStream> {
    let description = description(&token.description);
    let token_name = format_ident!("{}", name.to_shouty_snake_case());
    let token_value = format!("{schema_id}#{name}");
    Ok(quote! {
        #description
        pub const #token_name: &str = #token_value;
    })
}

fn lex_object(object: &LexObject, name: &str) -> Result<TokenStream> {
    let description = description(&object.description);
    let derives = derives()?;
    let struct_name = format_ident!("{}Data", name.to_pascal_case());
    let object_name = format_ident!("{}", name.to_pascal_case());
    let mut required = if let Some(required) = &object.required {
        HashSet::from_iter(required)
    } else {
        HashSet::new()
    };
    if let Some(nullable) = &object.nullable {
        for key in nullable {
            required.remove(&key);
        }
    }
    let mut fields = Vec::new();
    for key in object.properties.keys().sorted() {
        fields.push(lex_object_property(
            &object.properties[key],
            key,
            required.contains(key),
            name,
        )?);
    }
    Ok(quote! {
        use atrium_api::types::TryFromUnknown;

        #description
        #derives
        #[serde(rename_all = "camelCase")]
        pub struct #struct_name {
            #(#fields)*
        }

        pub type #object_name = atrium_api::types::Object<#struct_name>;

        impl From<atrium_api::types::Unknown> for RecordData {
            fn from(value: atrium_api::types::Unknown) -> Self {
                //TODO handle unwrap
                Self::try_from_unknown(value).unwrap()
            }
        }
    })
}

fn lex_object_property(
    property: &LexObjectProperty,
    name: &str,
    is_required: bool,
    object_name: &str,
) -> Result<TokenStream> {
    let (description, mut field_type) = match property {
        LexObjectProperty::Ref(r#ref) => ref_type(r#ref)?,
        LexObjectProperty::Union(union) => union_type(
            union,
            format!(
                "{}{}Refs",
                object_name.to_pascal_case(),
                name.to_pascal_case()
            )
            .as_str(),
        )?,
        LexObjectProperty::Bytes(bytes) => bytes_type(bytes)?,
        LexObjectProperty::CidLink(cid_link) => cid_link_type(cid_link)?,
        LexObjectProperty::Array(array) => array_type(array, name, Some(object_name))?,
        LexObjectProperty::Blob(blob) => blob_type(blob)?,
        LexObjectProperty::Boolean(boolean) => boolean_type(boolean)?,
        LexObjectProperty::Integer(integer) => integer_type(integer)?,
        LexObjectProperty::String(string) => string_type(string)?,
        LexObjectProperty::Unknown(unknown) => unknown_type(unknown)?,
    };
    let field_name = format_ident!(
        "{}",
        if name == "ref" || name == "type" {
            format!("r#{name}")
        } else {
            name.to_snake_case()
        }
    );
    let mut attributes = match property {
        LexObjectProperty::Bytes(_) => {
            let default = if is_required {
                quote!()
            } else {
                quote!(#[serde(default)])
            };
            quote! {
                #default
                #[serde(with = "serde_bytes")]
            }
        }
        _ => quote!(),
    };
    if !is_required {
        field_type = quote!(core::option::Option<#field_type>);
        attributes = quote! {
            #attributes
            #[serde(skip_serializing_if = "core::option::Option::is_none")]
        };
    }
    Ok(quote! {
        #description
        #attributes
        pub #field_name: #field_type,
    })
}

fn lex_string(string: &LexString, name: &str) -> Result<TokenStream> {
    let description = description(&string.description);
    let string_name = format_ident!("{}", name.to_pascal_case());
    Ok(quote! {
        #description
        pub type #string_name = String;
    })
}

fn ref_type(r#ref: &LexRef) -> Result<(TokenStream, TokenStream)> {
    let description = description(&r#ref.description);
    Ok((description, resolve_path(&r#ref.r#ref, "main")?))
}

fn union_type(union: &LexRefUnion, enum_name: &str) -> Result<(TokenStream, TokenStream)> {
    let description = description(&union.description);
    let enum_type_name = format_ident!("{}", enum_name);
    if union.closed.unwrap_or_default() {
        Ok((description, quote!(#enum_type_name)))
    } else {
        Ok((
            description,
            quote!(atrium_api::types::Union<#enum_type_name>),
        ))
    }
}

fn bytes_type(bytes: &LexBytes) -> Result<(TokenStream, TokenStream)> {
    let description = description(&bytes.description);
    Ok((description, quote!(Vec<u8>)))
}

fn cid_link_type(cid_link: &LexCidLink) -> Result<(TokenStream, TokenStream)> {
    let description = description(&cid_link.description);
    Ok((description, quote!(atrium_api::types::CidLink)))
}

fn array_type(
    array: &LexArray,
    name: &str,
    object_name: Option<&str>,
) -> Result<(TokenStream, TokenStream)> {
    let description = description(&array.description);
    let (_, item_type) = match &array.items {
        LexArrayItem::Integer(integer) => integer_type(integer)?,
        LexArrayItem::String(string) => string_type(string)?,
        LexArrayItem::Unknown(unknown) => unknown_type(unknown)?,
        LexArrayItem::CidLink(cid_link) => cid_link_type(cid_link)?,
        LexArrayItem::Ref(r#ref) => ref_type(r#ref)?,
        LexArrayItem::Union(union) => union_type(
            union,
            format!(
                "{}{}Item",
                object_name.map_or(String::new(), str::to_pascal_case),
                name.to_pascal_case()
            )
            .as_str(),
        )?,
        _ => unimplemented!("{:?}", array.items),
    };
    Ok((description, quote!(Vec<#item_type>)))
}

fn blob_type(blob: &LexBlob) -> Result<(TokenStream, TokenStream)> {
    let description = description(&blob.description);
    Ok((description, quote!(atrium_api::types::BlobRef)))
}

fn boolean_type(boolean: &LexBoolean) -> Result<(TokenStream, TokenStream)> {
    let description = description(&boolean.description);
    Ok((description, quote!(bool)))
}

fn integer_type(integer: &LexInteger) -> Result<(TokenStream, TokenStream)> {
    let description = description(&integer.description);
    let typ = match integer.minimum {
        // If the minimum acceptable value is 0, use the unsigned integer primitives, with
        // newtype wrappers enforcing the maximum acceptable value if relevant.
        Some(0) => match integer.maximum {
            // If a maximum acceptable value is specified, use the smallest fixed-width
            // unsigned type that can fit all acceptable values.
            Some(max) => match max {
                0x0000_0000..=0x0000_00fe => {
                    let max = max as u8;
                    quote!(atrium_api::types::LimitedU8<#max>)
                }
                0x0000_00ff => quote!(u8),
                0x0000_0100..=0x0000_fffe => {
                    let max = max as u16;
                    quote!(atrium_api::types::LimitedU16<#max>)
                }
                0x0000_ffff => quote!(u16),
                0x0001_0000..=0xffff_fffe => {
                    let max = max as u32;
                    quote!(atrium_api::types::LimitedU32<#max>)
                }
                0xffff_ffff => quote!(u32),
                _ => {
                    let max = max as u64;
                    quote!(atrium_api::types::LimitedU64<#max>)
                }
            },
            // If no maximum acceptable value is specified, assume that the integer might
            // be an index into (or the length of) something stored in memory (e.g. byte
            // slices).
            None => quote!(usize),
        },
        // If the minimum acceptable value is 1, use the `NonZeroU*` types, with newtype
        // wrappers enforcing the maximum acceptable value if relevant.
        Some(1) => match integer.maximum {
            // If a maximum acceptable value is specified, use the smallest fixed-width
            // unsigned type that can fit all acceptable values.
            Some(max) => match max {
                0x0000_0000..=0x0000_00fe => {
                    let max = max as u8;
                    quote!(atrium_api::types::LimitedNonZeroU8<#max>)
                }
                0x0000_00ff => quote!(core::num::NonZeroU8),
                0x0000_0100..=0x0000_fffe => {
                    let max = max as u16;
                    quote!(atrium_api::types::LimitedNonZeroU16<#max>)
                }
                0x0000_ffff => quote!(core::num::NonZeroU16),
                0x0001_0000..=0xffff_fffe => {
                    let max = max as u32;
                    quote!(atrium_api::types::LimitedNonZeroU32<#max>)
                }
                0xffff_ffff => quote!(core::num::NonZeroU32),
                _ => {
                    let max = max as u64;
                    quote!(atrium_api::types::LimitedNonZeroU64<#max>)
                }
            },
            None => quote!(core::num::NonZeroU64),
        },
        // For all other positive minimum acceptable values, use the `NonZeroU*` types
        // with newtype wrappers enforcing the minimum and maximum acceptable values.
        Some(min) if !min.is_negative() => match integer.maximum {
            // If a maximum acceptable value is specified, use the smallest fixed-width
            // unsigned type that can fit all acceptable values.
            Some(max) => match max {
                0x0000_0000..=0x0000_00ff => {
                    let min = min as u8;
                    let max = max as u8;
                    quote!(atrium_api::types::BoundedU8<#min, #max>)
                }
                0x0000_0100..=0x0000_ffff => {
                    let min = min as u16;
                    let max = max as u16;
                    quote!(atrium_api::types::BoundedU16<#min, #max>)
                }
                0x0001_0000..=0xffff_ffff => {
                    let min = min as u32;
                    let max = max as u32;
                    quote!(atrium_api::types::BoundedU32<#min, #max>)
                }
                _ => {
                    let min = min as u64;
                    let max = max as u64;
                    quote!(atrium_api::types::BoundedU64<#min, #max>)
                }
            },
            None => {
                let min = min as u64;
                quote!(atrium_api::types::BoundedU64<#min, u64::MAX>)
            }
        },
        // Use a signed integer type to represent a potentially negative Lexicon integer.
        Some(min) => match integer.maximum {
            // If a maximum acceptable value is specified, use the smallest fixed-width
            // signed type that can fit all acceptable values.
            Some(max) => match (min, max) {
                (-0x0000_0080, 0x0000_007f) => quote!(i8),
                (-0x0000_8000, 0x0000_7fff) => quote!(i16),
                (-0x8000_0000, 0x7fff_ffff) => quote!(i32),
                (i64::MIN, i64::MAX) => quote!(i64),
                // TODO: Implement newtype wrappers for bounded signed integers.
                _ => unimplemented!("i64(min: {}, max: {})", min, max),
            },
            None => quote!(i64),
        },
        None => match integer.maximum {
            Some(max) => unimplemented!("i64(max: {})", max),
            None => quote!(i64),
        },
    };
    Ok((description, typ))
}

fn string_type(string: &LexString) -> Result<(TokenStream, TokenStream)> {
    let description = description(&string.description);
    let typ = match string.format {
        Some(LexStringFormat::AtIdentifier) => quote!(atrium_api::types::string::AtIdentifier),
        Some(LexStringFormat::Cid) => quote!(atrium_api::types::string::Cid),
        Some(LexStringFormat::Datetime) => quote!(atrium_api::types::string::Datetime),
        Some(LexStringFormat::Did) => quote!(atrium_api::types::string::Did),
        Some(LexStringFormat::Handle) => quote!(atrium_api::types::string::Handle),
        Some(LexStringFormat::Nsid) => quote!(atrium_api::types::string::Nsid),
        Some(LexStringFormat::Language) => quote!(atrium_api::types::string::Language),
        Some(LexStringFormat::Tid) => quote!(atrium_api::types::string::Tid),
        Some(LexStringFormat::RecordKey) => quote!(atrium_api::types::string::RecordKey),
        // TODO: other formats (uri, at-uri)
        _ => quote!(String),
    };
    Ok((description, typ))
}

fn unknown_type(unknown: &LexUnknown) -> Result<(TokenStream, TokenStream)> {
    let description = description(&unknown.description);
    let typ = quote!(atrium_api::types::Unknown);
    Ok((description, typ))
}

fn description(description: &Option<String>) -> TokenStream {
    if let Some(description) = description {
        quote!(#[doc = #description])
    } else {
        quote!()
    }
}

fn refs_enum(refs: &[String], name: &str, schema_id: Option<&str>) -> Result<TokenStream> {
    enum_common(refs, name, schema_id, &[])
}

pub fn enum_common(
    refs: &[String],
    name: &str,
    schema_id: Option<&str>,
    namespaces: &[(String, Option<&str>)],
) -> Result<TokenStream> {
    let is_record = schema_id.is_none();
    let derives = derives()?;
    let enum_name = format_ident!("{name}");
    let mut variants = Vec::new();
    for r#ref in refs {
        let path = resolve_path(r#ref, if is_record { "record" } else { "main" })?;
        let rename = if r#ref.starts_with('#') {
            format!(
                "{}{}",
                schema_id.expect("schema id must be specified"),
                r#ref
            )
        } else {
            r#ref.clone()
        };
        let s = path.to_string().replace(' ', "");
        let mut parts = s
            .strip_prefix("crate::")
            .unwrap_or(&s)
            .split("::")
            .map(str::to_pascal_case)
            .collect_vec();
        if is_record {
            parts.pop();
        }
        let name = format_ident!("{}", parts.join(""));
        let mut feature = quote!();
        if is_record {
            if let Some((_, Some(feature_name))) = namespaces
                .iter()
                .find(|(prefix, _)| r#ref.starts_with(prefix))
            {
                feature = quote! {
                    #[cfg_attr(docsrs, doc(cfg(feature = #feature_name)))]
                    #[cfg(feature = #feature_name)]
                };
            }
        }
        variants.push(quote! {
            #feature
            #[serde(rename = #rename)]
            #name(Box<#path>)
        });
    }
    Ok(quote! {
        #derives
        #[serde(tag = "$type")]
        pub enum #enum_name {
            #(#variants),*
        }
    })
}

pub fn impl_into_record(
    refs: &[String],
    namespaces: &[(String, Option<&str>)],
) -> Result<TokenStream> {
    let mut impls = Vec::new();
    for r#ref in refs {
        let record_path = resolve_path(r#ref, "record")?;
        let record_data_path = resolve_path(r#ref, "record_data")?;
        let s = record_path.to_string().replace(' ', "");
        let mut parts = s
            .strip_prefix("crate::")
            .unwrap_or(&s)
            .split("::")
            .map(str::to_pascal_case)
            .collect_vec();
        parts.pop();
        let name = format_ident!("{}", parts.join(""));
        let mut feature = quote!();
        if let Some((_, Some(feature_name))) = namespaces
            .iter()
            .find(|(prefix, _)| r#ref.starts_with(prefix))
        {
            feature = quote! {
                #[cfg_attr(docsrs, doc(cfg(feature = #feature_name)))]
                #[cfg(feature = #feature_name)]
            };
        }
        impls.push(quote! {
            #feature
            impl From<#record_path> for KnownRecord {
                fn from(record: #record_path) -> Self {
                    KnownRecord::#name(Box::new(record))
                }
            }

            #feature
            impl From<#record_data_path> for KnownRecord {
                fn from(record_data: #record_data_path) -> Self {
                    KnownRecord::#name(Box::new(record_data.into()))
                }
            }

            impl Into<atrium_api::types::Unknown> for KnownRecord {
                fn into(self) -> atrium_api::types::Unknown {
                    atrium_api::types::TryIntoUnknown::try_into_unknown(&self).unwrap()
                }
            }
        });
    }
    Ok(quote!(#(#impls)*))
}

pub fn modules(
    names: &[String],
    components: &[&str],
    namespaces: &[(String, Option<&str>)],
) -> Result<TokenStream> {
    let v = names
        .iter()
        .map(|s| {
            let namespace = components.iter().chain(&[s.as_str()]).join(".");
            let feature = if let Some((_, Some(feature_name))) =
                namespaces.iter().find(|(prefix, _)| &namespace == prefix)
            {
                quote! {
                    #[cfg_attr(docsrs, doc(cfg(feature = #feature_name)))]
                    #[cfg(feature = #feature_name)]
                }
            } else {
                quote!()
            };
            let m = format_ident!("{s}");
            quote! {
                #feature
                pub mod #m;
            }
        })
        .collect_vec();
    Ok(quote!(#(#v)*))
}

pub fn lexicon_module(namespaces: &[(String, Option<&str>)]) -> Result<TokenStream> {
    let v = namespaces
        .iter()
        .filter_map(|s| {
            let possible_namespace = s.0.split('.').next();
            if possible_namespace.is_none() {
                panic!("unexpected namespace whole generating lexicon module");
            }
            let namespace = possible_namespace.unwrap();
            let m = format_ident!("{namespace}");
            quote! {
                pub mod #m;
            }
            .into()
        })
        .collect_vec();
    let top = quote! {
        pub mod record;
    };
    Ok(quote!(
        #top
        #(#v)*))
}

fn derives() -> Result<TokenStream> {
    let mut derives = Vec::new();
    for derive in &[
        "serde::Serialize",
        "serde::Deserialize",
        "Debug",
        "Clone",
        "PartialEq",
        "Eq",
    ] {
        derives.push(syn::parse_str::<Path>(derive)?);
    }
    Ok(quote!(#[derive(#(#derives),*)]))
}

fn resolve_path(r#ref: &str, default: &str) -> Result<TokenStream> {
    let (namespace, def) = r#ref.split_once('#').unwrap_or((r#ref, default));
    let path = syn::parse_str::<Path>(&if namespace.is_empty() {
        def.to_pascal_case()
    } else {
        format!(
            "crate::lexicons::{}::{}",
            namespace.split('.').map(str::to_snake_case).join("::"),
            if def.chars().all(char::is_uppercase) {
                def.to_string()
            } else {
                def.to_pascal_case()
            }
        )
    })?;
    Ok(quote!(#path))
}
