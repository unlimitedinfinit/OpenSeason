use std::path::PathBuf;

fn main() {
    load_dotenv_into_rustc_env();
    tauri_build::build();
}

/// Forward `OPENSEASON_FIREBASE_API_KEY` from a gitignored `.env` to rustc so
/// `option_env!` can bake it into release builds without putting the key in source.
fn load_dotenv_into_rustc_env() {
    println!("cargo:rerun-if-env-changed=OPENSEASON_FIREBASE_API_KEY");

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [manifest.join(".env"), manifest.join("..").join(".env")];
    for path in &candidates {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    if std::env::var("OPENSEASON_FIREBASE_API_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .is_some()
    {
        return;
    }

    for path in &candidates {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if let Some(value) = parse_dotenv_key(&text, "OPENSEASON_FIREBASE_API_KEY") {
            println!("cargo:rustc-env=OPENSEASON_FIREBASE_API_KEY={value}");
            return;
        }
    }
}

fn parse_dotenv_key(text: &str, want: &str) -> Option<String> {
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line).trim();
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != want {
            continue;
        }
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
            .unwrap_or(value)
            .trim();
        if value.is_empty() {
            return None;
        }
        return Some(value.to_string());
    }
    None
}
