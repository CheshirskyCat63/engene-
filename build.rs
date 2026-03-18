//! Build script — generates compile-time metadata for BuildManifest.

fn main() {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    println!("cargo:rustc-env=ENGENE_BUILD_TIME={}", timestamp);

    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=ENGENE_GIT_HASH={}", hash);
        }
    }

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=ENGENE_BUILD_PROFILE={}", profile);

    let mut features = Vec::new();
    for (key, _) in std::env::vars() {
        if let Some(feat) = key.strip_prefix("CARGO_FEATURE_") {
            features.push(feat.to_lowercase());
        }
    }
    features.sort();
    println!(
        "cargo:rustc-env=ENGENE_FEATURE_FLAGS={}",
        features.join(",")
    );

    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=PROFILE");
}
