use cmake;
use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .clang_arg("-Isrc/external/r3d/external/raylib/src")
        .header("src/external/r3d/include/r3d.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Could not create R3D binding.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let dst = cmake::Config::new("src/external/r3d")
        // do not generate documentation for R3D.
        .define("R3D_BUILD_DOCS", "0")
        // do not build any example for R3D.
        .define("R3D_BUILD_EXAMPLES", "0")
        .define("R3D_RAYLIB_VENDORED", "1")
        .define("R3D_ASSIMP_VENDORED", "1")
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    // research if we can use static instead of dylib?
    println!("cargo:rustc-link-lib=dylib=r3d");
}
