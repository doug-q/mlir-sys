use std::{
    env,
    str,
    error::Error,
    path::{PathBuf, Path},
    process::{exit, Command},
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
    let llvm_config_path = env::var("DEP_MLIR_CONFIG_PATH")?;
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let cargo_manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let wrapper_h = cargo_manifest_dir.join("wrapper.h");

    let cflags = llvm_config(&llvm_config_path, "--cflags")?;


    for l in link_libs.split(";").map(|x| format!("cargo:rustc-link-lib={}", x)) {
        println!("{}",l);
    }
    let builder = bindgen::builder()
        .header(wrapper_h.to_str().unwrap())
        .clang_args(include_dirs.split(";").chain(std::iter::once(cargo_manifest_dir.to_str().unwrap())).map(|x| format!("-I{}", x)))
        .clang_args(cflags.split(" "))
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
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-ifchanged={}", wrapper_h.display());

    Ok(())
}

fn llvm_config(config_path: &str, argument: &str) -> Result<String, Box<dyn Error>> {
    let prefix = Path::new(config_path).join("bin");
    let call = format!(
        "{} --link-shared {}",
        prefix.join("llvm-config").display(),
        argument
    );

    Ok(str::from_utf8(
        &if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &call]).output()?
        } else {
            Command::new("sh").arg("-c").arg(&call).output()?
        }
        .stdout,
    )?
    .trim()
    .to_string())
}
