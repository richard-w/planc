use std::process::Command;

fn main() {
    // Re-run this script if the frontend changes.
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/.browserslistrc");
    println!("cargo:rerun-if-changed=web/angular.json");
    println!("cargo:rerun-if-changed=web/build.sh");
    println!("cargo:rerun-if-changed=web/Dockerfile");
    println!("cargo:rerun-if-changed=web/package-lock.json");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/tsconfig.app.json");
    println!("cargo:rerun-if-changed=web/tsconfig.json");
    // Run the build script.
    Command::new("web/build.sh")
        .spawn()
        .expect("Failed to start build script");
}
