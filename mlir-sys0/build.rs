use std::{
    env,
    error::Error,
    path::Path,
    process::{exit, Command},
    str,
    vec::Vec,
};

const LLVM_MAJOR_VERSION: usize = 18;

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let version = llvm_config("--version")?;

    if !version.starts_with(&format!("{}.", LLVM_MAJOR_VERSION)) {
        return Err(format!(
            "failed to find correct version ({}.x.x) of llvm-config (found {})",
            LLVM_MAJOR_VERSION, version
        )
        .into());
    }

    if let Some(x) = get_config_path() {
        println!("cargo:config_path={}", x);
    }
    println!("cargo:include_dirs={}", llvm_config("--includedir")?);
    println!("cargo:library_name=MLIR-C");
    println!("cargo:rerun-if-env-changed={}", get_config_var());
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rustc-link-search=native={}", llvm_config("--libdir")?);
    println!("cargo:rustc-link-lib=dylib=MLIR-C");

    let mut libs = llvm_config("--system-libs")?.trim().split(' ').filter(|x| !x.is_empty()).map(|flag| {
        let flag = flag.trim_start_matches("-l");

        if flag.starts_with('/') {
            // llvm-config returns absolute paths for dynamically linked libraries.
            let path = Path::new(flag);

            println!(
                "cargo:rustc-link-search={}",
                path.parent().unwrap().display()
            );
            format!(
                "cargo:rustc-link-lib={}",
                path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split_once('.')
                    .unwrap()
                    .0
                    .trim_start_matches("lib")
            )
        } else {
            format!("cargo:rustc-link-lib={}", flag)
        }
    }).collect::<Vec<_>>();
    if let Some(name) = get_system_libcpp() {
        libs.push(name.to_string());
    }

    println!("cargo:link_libs={}", libs.join(";"));
    for l in libs {
        println!("cargo:rustc-link-lib=dylib={}", l);
    }

    Ok(())
}

fn get_system_libcpp() -> Option<&'static str> {
    if cfg!(target_env = "msvc") {
        None
    } else if cfg!(target_os = "macos") {
        Some("c++")
    } else {
        Some("stdc++")
    }
}

fn get_config_var() -> String {
    format!("MLIR_SYS_{}0_PREFIX", LLVM_MAJOR_VERSION)
}

fn get_config_path() -> Option<String> {
    env::var(format!("MLIR_SYS_{}0_PREFIX", LLVM_MAJOR_VERSION)).ok()
}

fn llvm_config(argument: &str) -> Result<String, Box<dyn Error>> {
    let prefix = get_config_path()
        .map(|path| Path::new(&path).join("bin"))
        .unwrap_or_default();
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
