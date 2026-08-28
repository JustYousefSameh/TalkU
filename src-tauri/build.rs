use std::path::PathBuf;

fn copy_wireguard_dll() {
    // defguard_wireguard_rs loads the WireGuard DLL from the relative path
    // "resources-windows/binaries/wireguard-amd64.dll" (measured from the
    // process working directory, which for `cargo tauri dev` / packaged runs is
    // the target profile directory). Copy the DLL we ship (in
    // `CARGO_MANIFEST_DIR/binaries`) into that location on every build.
    let source = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("binaries")
        .join("wireguard-amd64.dll");
    if !source.exists() {
        println!("cargo:warning=wireguard-amd64.dll not present at {:?}; skipping copy", source);
        return;
    }

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("..")
                .join("target")
        });

    let dest_dir = target_dir
        .join(&profile)
        .join("resources-windows")
        .join("binaries");
    let dest = dest_dir.join("wireguard-amd64.dll");

    if std::fs::create_dir_all(&dest_dir).is_err() {
        println!("cargo:warning=failed to create {:?}", dest_dir);
        return;
    }
    match std::fs::copy(&source, &dest) {
        Ok(_) => println!("cargo:warning=copied wireguard-amd64.dll -> {:?}", dest),
        Err(e) => println!("cargo:warning=failed to copy wireguard-amd64.dll: {e}"),
    }
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        copy_wireguard_dll();
    }
    tauri_build::build()
}
