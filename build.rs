use std::env;
use std::path::PathBuf;

use bindgen::{Builder, CargoCallbacks, EnumVariation};

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.hpp");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("sdk/wrapper.hpp")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(CargoCallbacks))
        // Don't generate layout tests.
        .layout_tests(false)
        // Don't generate derives.
        .derive_copy(false)
        .derive_debug(false)
        .derive_default(false)
        .derive_eq(false)
        .derive_hash(false)
        .derive_ord(false)
        .derive_partialeq(false)
        .derive_partialord(false)
        .impl_debug(false)
        .impl_partialeq(false)
        // Blacklist C types that we plan to replace in Rust.
        .blacklist_type("vec3_t")
        // To prune unused bindings, explicitly declare the types we care about.
        // bindgen will produce these types and their dependencies.
        .whitelist_type("cl_entity_s")
        .whitelist_type("cl_enginefuncs_s")
        .whitelist_type("cl_clientfuncs_s")
        .whitelist_type("user_msg_s")
        .whitelist_type("GLenum")
        // Format
        .rustfmt_bindings(true)
        // Use Rust enums instead of constants to represent enum variants.
        .default_enum_style(EnumVariation::Rust { non_exhaustive: true })
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/sdk.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("sdk.rs"))
        .expect("Couldn't write bindings!");
}