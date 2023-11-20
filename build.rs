use std::{
    env,
    error::Error,
    path::{PathBuf},
    process::{exit},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let link_libs = env::var("DEP_MLIR_LINK_LIBS")?;
    let include_dirs = env::var("DEP_MLIR_INCLUDE_DIRS")?;
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    println!("cargo:rerun-if-env-changed=DEP_MLIR_INCLUDE_DIRS");
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-ifchanged=wrapper.h");
    for l in link_libs.split(";").map(|x| format!("cargo:rustc-link-lib={}", x)) {
        println!("{}",l);
    }
    let builder = bindgen::builder()
        .header("wrapper.h")
        .clang_args(include_dirs.split(";").map(|x| format!("-I{}", x)))
        .wrap_static_fns(true)
        .wrap_static_fns_path(out_dir.join("mlir_extern.c"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    builder.generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))?;

    cc::Build::new()
        .file(out_dir.join("mlir_extern.c"))
        .includes(include_dirs.split(";"))
        .compile("mlir_extern");
    println!("cargo:rustc-link-lib=static=mlir_extern");

    Ok(())
}
