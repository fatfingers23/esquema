
# esquema examples
Both projects use the [Statusphere](./lexicons/status.json) as the example to generate Rust types and use them to add a new record and list the last 3 statuses you have.

## Setup
To run the example make sure to copy [.env.template](./esquema-example/.env.template) as `.env` in the location you run the example. Then make sure to fill it out. It is recommended to use a Bluesky app password and not your normal password
for this.


## Code generation with the CLI tool
[cli_example](./src/bin/cli_example.rs) is an example  using the [esquema-cli](../esquema-cli/) to create and read records. The benefit of this is it places the code in your project as part of the source code. But is a manual process if you change the lexicon schema.

You can generate the Rust types by running `cargo run --bin esquema-cli -- generate` from root, the args are defaulted
to work for the example project. This will then take the lexicons found
in [./lexicons](./lexicons) and generate Rust types from them in the
folder [./src/lexicons](./src/lexicons). Then it's as easy as putting a `mod lexicons;` at the top of your `main.rs` (for this example we have it in the [lib.rs](./src/lib.rs))



## Code generation with build.rs
This generates the Rust types on every build to the out directory so it's automatic and out of your source code. I'm still researching ways to make sure this only runs when the lexicons change, currently it jus regenerates the rust code on every build. A bit longer build time for the convince.

You can check the [build.rs](./build.rs) to see an example on how you can generate the types and put them in the `OUT` directory which is usually found in `target/debug/build/esquema-example-{hash}/out`

then in [build_example.rs](./src/bin/build_example.rs) we use the following code to dynamically import the modules.

```rust
mod lexicons {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

```

---

The module name is hard coded at this moment for any code generation but plans to have it set
from the folder name you set as output. 


Try changing the [status.json](./lexicons/status.json) around
and see it generate new types!

