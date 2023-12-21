use std::env;
use std::path::PathBuf;
const HEADER: &str =  "../bass_synth/include/messages.hpp";
const OUTPUT: &str = "src/bindings.rs";

fn main() {
    println!("cargo:rerun-if-changed={}", HEADER);

    let bindings = bindgen::Builder::default()
        .header(HEADER)
        .generate()
        .expect("Failed to generate wrappers");

    //    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(OUTPUT)
        .expect("Couldn't write bindings!");

}