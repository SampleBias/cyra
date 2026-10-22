use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    let compiler = std::env::var_os("RUSTC").expect("Cargo supplies RUSTC");
    let output = Command::new(compiler)
        .arg("--version")
        .output()
        .expect("read compiler version");
    assert!(output.status.success(), "compiler version probe failed");
    let version = String::from_utf8(output.stdout).expect("UTF-8 compiler version");
    println!("cargo:rustc-env=CYRA_BUILD_RUSTC={}", version.trim());
    println!(
        "cargo:rustc-env=CYRA_BUILD_TARGET={}",
        std::env::var("TARGET").expect("Cargo supplies TARGET")
    );
    println!(
        "cargo:rustc-env=CYRA_BUILD_PROFILE={}",
        std::env::var("PROFILE").expect("Cargo supplies PROFILE")
    );
}
