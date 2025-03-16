# esquema

Rust tools for interacting with custom ATProto Lexicons.
This is a direct fork of [atrium-codegen](https://github.com/sugyan/atrium/tree/main/lexicon/atrium-codegen), any code
generation and parsing of the JSON Lexicon records are thanks to that project and their efforts.

The main goal of this project is to allow you to have strong types for your custom Lexicon records and be able to
validate them.

# Goals

- [ ] Generate Rust types from JSON Lexicon files and be able to use them in atrium's `com.atproto.repo.*`'s record
  methods easily via CLI
- [ ] Generate Rust types the same way but in a `build.rs`
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

A command line tool to help you generate Rust types from lexicon definitions

## [esquema-codegen](./esquema-codegen)

A fork of [atrium-codegen](https://github.com/sugyan/atrium/tree/main/lexicon/atrium-codegen) to generate the Rust types
in a way that can be used by other projects

## [esquema-example](./esquema-example)

An example project using the Statusphere lexicon with the code generated from `esquema-cli` to create and read records

## [esquema-validator](./esquema-validator)

Future. This will be a crate to help you validate custom lexicon schemas to make sure the record is valid according to
the lexicon schema