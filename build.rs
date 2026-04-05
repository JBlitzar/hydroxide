fn main() {
    // OUT_DIR contains the actual profile directory name, e.g. .../target/extra/build/...
    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    let effective_profile = if out_dir.contains("/extra/") {
        "extra"
    } else if out_dir.contains("/iteration/") {
        "iteration"
    } else if out_dir.contains("/release/") {
        "release"
    } else {
        "debug"
    };

    println!("cargo:rustc-env=OXIDE_PROFILE={effective_profile}");
}
