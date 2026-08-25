//! Build script.
//!
//! Three jobs beyond the usual Tauri codegen:
//!
//!   * embed the application manifest, which is what actually requests elevation
//!     (Full Tunnel needs an elevated token to touch the route table);
//!   * read the core's version out of `core/aether/Cargo.toml` and expose it as
//!     `AETHER_CORE_VERSION`, so the About page can never show a stale number;
//!   * fail early with a readable message if the core is missing, rather than
//!     letting Cargo emit a wall of unresolved-path errors.

use std::path::Path;

fn main() {
    check_core_present();
    export_core_version();

    // Whether to embed the manifest depends on the *target*, not the host. Using
    // `cfg!(windows)` here would test the machine running the build script, so a
    // cross-compiled Windows binary would silently ship without its manifest — and
    // therefore without the elevation request that Full Tunnel mode needs.
    let targets_windows = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows");

    if targets_windows {
        let attributes =
            tauri_build::WindowsAttributes::new().app_manifest(include_str!("aether-gui.manifest"));
        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(attributes))
            .expect("failed to build with the embedded manifest");
    } else {
        tauri_build::build();
    }
}

/// The core is a path dependency, so a missing clone produces a confusing error.
fn check_core_present() {
    let core = Path::new("../core/aether/Cargo.toml");
    if !core.exists() {
        panic!(
            "\n\nThe Aether core is missing. Clone it next to this project:\n\n    \
             git clone --depth 1 https://github.com/CluvexStudio/Aether.git core\n\n\
             It must end up at core/aether and core/quiche.\n"
        );
    }
}

fn export_core_version() {
    let manifest = Path::new("../core/aether/Cargo.toml");
    println!("cargo:rerun-if-changed=../core/aether/Cargo.toml");

    let version = std::fs::read_to_string(manifest)
        .ok()
        .and_then(|text| parse_version(&text))
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=AETHER_CORE_VERSION={version}");
}

/// Pull `version = "x.y.z"` out of the `[package]` section.
///
/// Stops at the next section header, so a dependency's version can never be picked
/// up by mistake.
fn parse_version(manifest: &str) -> Option<String> {
    let mut in_package = false;

    for line in manifest.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }

        if !in_package {
            continue;
        }

        if let Some(rest) = line.strip_prefix("version") {
            return rest
                .trim_start()
                .strip_prefix('=')
                .map(|value| value.trim().trim_matches('"').to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::parse_version;

    #[test]
    fn the_package_version_is_read() {
        let manifest = "[package]\nname = \"aether\"\nversion = \"1.7.0\"\n";
        assert_eq!(parse_version(manifest).as_deref(), Some("1.7.0"));
    }

    #[test]
    fn a_dependency_version_is_not_mistaken_for_the_package_one() {
        let manifest = "[package]\nname = \"aether\"\n\n[dependencies]\nlog = \"0.4\"\n";
        assert_eq!(parse_version(manifest), None);
    }
}
