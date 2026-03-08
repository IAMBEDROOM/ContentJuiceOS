use std::path::PathBuf;

fn main() {
    // Load .env file for compile-time environment variables (Twitch OAuth credentials, etc.)
    load_dotenv();

    tauri_build::build();

    // In dev mode, Tauri's shell plugin resolves sidecars relative to the app executable
    // directory (target/debug/ or target/release/) using just the name + ".exe" on Windows.
    // The externalBin entry "binaries/socket-io-server" means it looks for
    // target/{profile}/binaries/socket-io-server.exe (without the platform triple).
    //
    // The build.mjs script places the binary in src-tauri/binaries/ with the triple suffix
    // (e.g., socket-io-server-x86_64-pc-windows-msvc.exe) for production bundling.
    // We copy it here without the triple so it works in dev mode too.
    let target_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf();

    let source_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("binaries");
    let dest_dir = target_dir.join("binaries");

    if source_dir.exists() {
        let _ = std::fs::create_dir_all(&dest_dir);
        if let Ok(entries) = std::fs::read_dir(&source_dir) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.is_file() {
                    // Copy with original name (for production compatibility)
                    let dest = dest_dir.join(entry.file_name());
                    let _ = std::fs::copy(&src, &dest);

                    // Also copy without the platform triple for dev mode resolution.
                    // e.g., "socket-io-server-x86_64-pc-windows-msvc.exe" -> "socket-io-server.exe"
                    let filename = entry.file_name();
                    let name = filename.to_string_lossy();
                    if let Some(base) = strip_target_triple(&name) {
                        let dev_dest = dest_dir.join(base);
                        let _ = std::fs::copy(&src, &dev_dest);
                    }
                }
            }
        }
    }
}

/// Load environment variables from a `.env` file in the manifest directory.
/// This enables `env!("TWITCH_CLIENT_ID")` etc. at compile time without
/// requiring a runtime dotenv crate.
fn load_dotenv() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let env_file = manifest_dir.join(".env");

    if !env_file.exists() {
        // Not an error — CI uses actual env vars, not .env files
        return;
    }

    // Re-run build script if .env changes
    println!("cargo:rerun-if-changed=.env");

    let contents = std::fs::read_to_string(&env_file).expect("Failed to read .env file");
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            // Only set if not already set (real env vars take precedence)
            if std::env::var(key).is_err() {
                println!("cargo:rustc-env={key}={value}");
            }
        }
    }
}

/// Strip target triple from a sidecar filename.
/// "socket-io-server-x86_64-pc-windows-msvc.exe" -> "socket-io-server.exe"
fn strip_target_triple(filename: &str) -> Option<String> {
    let triples = [
        "-x86_64-pc-windows-msvc",
        "-aarch64-pc-windows-msvc",
        "-x86_64-apple-darwin",
        "-aarch64-apple-darwin",
        "-x86_64-unknown-linux-gnu",
        "-aarch64-unknown-linux-gnu",
    ];
    for triple in &triples {
        if let Some(pos) = filename.find(triple) {
            let base = &filename[..pos];
            let rest = &filename[pos + triple.len()..];
            return Some(format!("{base}{rest}"));
        }
    }
    None
}
