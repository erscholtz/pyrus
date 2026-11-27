use std::process::Command;

// Bakes the current commit's short sha into the binary as `PYRUS_GIT_SHA` so the
// renderer can stamp it into the PDF metadata.
fn main() {
    let sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|sha| sha.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=PYRUS_GIT_SHA={sha}");

    // rebuild when HEAD moves (checkout, new commit) instead of on every build
    println!("cargo:rerun-if-changed=.git/HEAD");
    if let Ok(head) = std::fs::read_to_string(".git/HEAD") {
        if let Some(reference) = head.trim().strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed=.git/{reference}");
        }
    }
    println!("cargo:rerun-if-changed=.git/packed-refs");
}
