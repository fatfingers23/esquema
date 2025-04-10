> # esquema
> /es'kema/ \
> layout , scheme

> [!WARNING]
> ***This project is a work in progress and in the early stages of development. Many items may not work or even been
implemented yet***

Rust tools for interacting with custom ATProto Lexicons.
This is a direct fork of [atrium-codegen](https://github.com/sugyan/atrium/tree/main/lexicon/atrium-codegen), any code
generation and parsing of the JSON Lexicon records are thanks to that project and their efforts.

# End Goal

You should be able to easily use strong Rust types from ATProto lexicon schemas to build out your Rust project that uses
ATProto records.

You should be able to validate those records structure and content against that schema. If your app depends on a field
being there or being a certain length you should be able to easily check that with out manually doing it.

# Todo list

- [x] Generate Rust types from JSON Lexicon files and be able to use them in atrium's `com.atproto.repo.*`'s record
  methods easily via CLI
- [x] Generate Rust types the same way but in a `build.rs`
- [x] Generate Rust types from a passed
  in [LexiconDoc](https://github.com/sugyan/atrium/blob/f162f815a04b5ecb0421b390d521c883c41d5f75/lexicon/atrium-lex/src/lib.rs#L16)
- [x] Generate Rust types from remote did lexicon schema ATProto records
- [ ] Custom Lexicon validation of data
- [ ] Helpers for writing valid DNS TXT records and ATProto records so your Lexicons are public and valid
- [ ] A way to write Rust types with being descriptive to generate Lexicon files

# Why?

I've been playing around with ATProto records after checking out
the [Statusphere example app](https://atproto.com/guides/applications), I'd like to recreate this tutorial in Rust and
have a way to generate the Lexicon records in a type-safe way. Along the way I had a lot of issues with validating and
in general creating custom records. So I am hoping this project will help others with an easier API to validate ATProto
records and create them when using rust

Some problems these crates will try to solve

- Rust code generation from lexicon files for Rust types
- Serializing and deserializing those Rust ATProto record types easily for use in atrium
- It does not appear Bluesky currently support custom lexicon scheme resolution and validation so to give you the tools
  in Rust to know if the record is valid according to the schema.

## [esquema-cli](./esquema-cli)

A command line tool to help you generate Rust types from lexicon definitions,
Check [esquema-example](#esquema-example) for an example on how to run the command in a projects setting.

Some examples:

create types from local lexicon schema JSON files
`esquema-cli generate local --lexdir ./esquema-example/lexicons/ --outdir ./esquema-example/src/lexicons/`

Create types from a remote ATProtocol record with a lexicon schema. Using statusphere as the example
`esquema-cli generate remote --handle statusphere.xyz --namespace xyz --outdir ./esquema-example/src/lexicons`

## [esquema-codegen](./esquema-codegen)

A fork of [atrium-codegen](https://github.com/sugyan/atrium/tree/main/lexicon/atrium-codegen) to generate the Rust types
in a way that can be used by other projects like how
in [atrium-api](https://github.com/sugyan/atrium/tree/main/atrium-api/src) is used for Bluesky's lexicons.

## [esquema-example](./esquema-example)

An example project show casing how to use esquema to generate Rust types from ATProto lexicon records
via [esquema-cli](./esquema-cli/) or using [esquema-codegen](./esquema-codegen/) to generate the types from
a [build.rs](./esquema-example/build.rs) file.

Check out the [readme](./esquema-example/README.md) for more info.

## [esquema-validator](./esquema-validator)

Future. This will be a crate to help you validate custom lexicon schemas to make sure the record is valid according to
the lexicon schema