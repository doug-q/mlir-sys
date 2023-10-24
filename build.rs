use std::{

    env,
    error::Error,
    path::Path,
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
    println!("cargo:rerun-if-env-changed=DEP_MLIR_INCLUDE_DIRS");
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-ifchanged=wrapper.h");
    for l in link_libs.split(";").map(|x| format!("cargo:rustc-link-lib={}", x)) {
        println!("{}",l);
    }
    let builder = bindgen::builder()
        .header("wrapper.h")
        .clang_args(include_dirs.split(";").map(|x| format!("-I{}", x)))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));
    builder.generate()
        .unwrap()
        .write_to_file(Path::new(&env::var("OUT_DIR")?).join("bindings.rs"))?;

    Ok(())
}
