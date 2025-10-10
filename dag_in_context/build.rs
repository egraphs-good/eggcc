use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn build_binary(source: &Path, output: &Path, extra_flags: &[&str]) {
    let tiger_dir = source
        .parent()
        .expect("source file should have a parent directory");

    let mut cmd = Command::new("g++");
    cmd.current_dir(tiger_dir)
        .args(["-std=c++17", "-O2", "-Wno-unused-result"])
        .args(extra_flags)
        .arg(source.file_name().expect("source file name"))
        .arg("-o")
        .arg(output);

    let status = cmd
        .status()
        .expect("failed to invoke C++ compiler for tiger binaries");

    if !status.success() {
        panic!(
            "failed to compile {:?} -> {:?} with status {}",
            source, output, status
        );
    }
}

fn binary_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let profile = env::var("PROFILE").expect("PROFILE not set");

    let tiger_dir = PathBuf::from(&manifest_dir).join("src/tiger");
    let target_dir = PathBuf::from(&manifest_dir)
        .join("..")
        .join("target")
        .join(&profile);

    fs::create_dir_all(&target_dir).expect("failed to create target directory");

    let json2egraph_out = target_dir.join(binary_name("json2egraph"));
    let tiger_out = target_dir.join(binary_name("tiger"));

    build_binary(&tiger_dir.join("json2egraph.cpp"), &json2egraph_out, &[]);
    build_binary(&tiger_dir.join("main.cpp"), &tiger_out, &["-DEMIT_JSON"]);

    if let Ok(entries) = fs::read_dir(&tiger_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "cpp" {
                    println!("cargo::rerun-if-changed={}", path.display());
                }
            }
        }
    }
}
