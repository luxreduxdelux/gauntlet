use std::env;
use std::path::PathBuf;

fn main() {
    // create R3D bind file.
    let bind = bindgen::Builder::default()
        .clang_arg("-Isrc/external/r3d/external/raylib/src")
        .header("src/external/r3d/include/r3d.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Could not create R3D bind file.");

    let path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bind.write_to_file(path)
        .expect("Could not write R3D bind file.");

    //================================================================

    // build R3D library and link with it.
    let path = cmake::Config::new("src/external/r3d")
        // do not generate documentation for R3D.
        .define("R3D_BUILD_DOCS", "0")
        // do not build any example for R3D.
        .define("R3D_BUILD_EXAMPLES", "0")
        // use built-in raylib sub-module.
        .define("R3D_RAYLIB_VENDORED", "1")
        // use built-in assimp sub-module.
        .define("R3D_ASSIMP_VENDORED", "1")
        .build();

    // research if we can use static instead of dylib?
    println!("cargo:rustc-link-search=native={}/build", path.display());
    println!("cargo:rustc-link-lib=dylib=r3d");
}
