use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn build_binary(tiger_dir: &Path, sources: &[&str], output: &Path, extra_flags: &[&str]) {
    let mut cmd = Command::new("g++");
    cmd.current_dir(tiger_dir)
        .args(["-std=c++17", "-O3", "-Wno-unused-result"])
        .args(extra_flags);

    for source in sources {
        cmd.arg(source);
    }

    cmd.arg("-o").arg(output);

    let status = cmd
        .status()
        .expect("failed to invoke C++ compiler for tiger binaries");

    if !status.success() {
        panic!(
            "failed to compile {:?} -> {:?} with status {}",
            sources, output, status
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

    let tiger_out = target_dir.join(binary_name("tiger"));

    build_binary(
        &tiger_dir,
        &[
            "egraphin.cpp",
            "greedy.cpp",
            "json2egraphin.cpp",
            "ilp.cpp",
            "time_ilp.cpp",
            "main.cpp",
            "regionalize.cpp",
            "statewalkdp.cpp",
            "tiger.cpp",
            "toegglog.cpp",
            "debug.cpp",
        ],
        &tiger_out,
        &["-DEMIT_JSON"],
    );

    if let Ok(entries) = fs::read_dir(&tiger_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "cpp" || ext == "h" {
                    println!("cargo::rerun-if-changed={}", path.display());
                }
            }
        }
    }

    build_tiger_rs(&manifest_dir, &target_dir, &profile);
}

fn build_tiger_rs(manifest_dir: &str, target_dir: &Path, profile: &str) {
    let tiger_rs_crate = PathBuf::from(manifest_dir).join("..").join("tiger");
    if !tiger_rs_crate.join("Cargo.toml").is_file() {
        return;
    }

    // Build into the tiger crate's own target dir to avoid contending with the
    // parent cargo's lock on this workspace's target dir, then copy the binary
    // into place where find_tiger_binary() expects it.
    let nested_target = tiger_rs_crate.join("target");
    let cargo_bin = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let mut cmd = Command::new(cargo_bin);
    cmd.current_dir(&tiger_rs_crate)
        .args(["build", "--bin", "tiger-rs"]);
    // The tiger crate is a separate helper-binary crate, not part of this workspace's
    // lint surface. When the parent build runs under `cargo clippy`, it exports a
    // clippy wrapper that would otherwise leak into this nested build and lint tiger
    // with `-D warnings`. Clear the wrapper so the nested build is a plain rustc build.
    cmd.env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("RUSTC_WRAPPER");
    if profile == "release" {
        cmd.arg("--release");
    }

    let status = cmd
        .status()
        .expect("failed to invoke cargo for tiger-rs binary");
    if !status.success() {
        panic!("cargo build for tiger-rs failed with status {}", status);
    }

    let src = nested_target.join(profile).join(binary_name("tiger-rs"));
    let dst = target_dir.join(binary_name("tiger-rs"));
    fs::copy(&src, &dst).unwrap_or_else(|err| {
        panic!(
            "failed to copy {} -> {}: {}",
            src.display(),
            dst.display(),
            err
        )
    });

    println!(
        "cargo::rerun-if-changed={}",
        tiger_rs_crate.join("Cargo.toml").display()
    );
    let src_dir = tiger_rs_crate.join("src");
    if let Ok(entries) = fs::read_dir(&src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "rs" {
                    println!("cargo::rerun-if-changed={}", path.display());
                }
            }
        }
    }
}
