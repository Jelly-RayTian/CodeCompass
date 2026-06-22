fn main() {
    tauri_build::build();

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={ts}");
}
