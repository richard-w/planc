use std::process::Command;

fn main() {
    // Build the frontend
    let frontend_build_exit = Command::new("trunk")
        .current_dir("web")
        .args(["build"])
        .status()
        .expect("failed to run frontend build");
    if !frontend_build_exit.success() {
        panic!(
            "building the frontend failed: {:?}",
            frontend_build_exit.code()
        );
    }
    // Rerun if frontend files change
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/styles");
    println!("cargo:rerun-if-changed=web/Cargo.toml");
    println!("cargo:rerun-if-changed=web/Cargo.lock");
    println!("cargo:rerun-if-changed=web/index.html");
}
