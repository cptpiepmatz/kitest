use std::{env, path::PathBuf, sync::LazyLock};

static HTML_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("doc")
        .join("html")
});

fn main() {
    // This forces rebuilds for `cargo doc` when files in doc/html changed.
    for entry in HTML_DIR.read_dir().unwrap() {
        let path = entry.unwrap().path();
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
