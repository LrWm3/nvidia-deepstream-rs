extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Setup GStreamer 
    let pk = pkg_config::Config::new().probe("gstreamer-1.0").unwrap();
    for path in &pk.link_paths {
        println!("cargo:rustc-link-search={:?}", path);
    }

    // Determine DeepStream Root Path
    // Logic: Check DEEPSTREAM_PATH env var -> Check default symlink -> Panic if neither works
    let ds_root = env::var("DEEPSTREAM_PATH")
        .unwrap_or_else(|_| "/opt/nvidia/deepstream/deepstream".to_string());

    let ds_include_path = PathBuf::from(&ds_root).join("sources/includes");

    // Verify path exists to give a helpful error message
    if !ds_include_path.exists() {
        panic!(
            "DeepStream headers not found at: {}. \
             Please set the DEEPSTREAM_PATH environment variable to your DeepStream installation root.",
            ds_include_path.display()
        );
    }

    // Rerun build if the env var changes
    println!("cargo:rerun-if-env-changed=DEEPSTREAM_PATH");

    // Configure Bindgen
    let mut bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Inject the dynamic path
        .clang_arg(format!("-I{}", ds_include_path.display()));

    // Add GStreamer includes
    for path in &pk.include_paths {
        bindings = bindings.clang_arg(format!("-I{}", path.to_str().unwrap()));
    }

    bindings = bindings.parse_callbacks(Box::new(bindgen::CargoCallbacks));

    let bindings = bindings.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
