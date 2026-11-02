/*
 * [Input] Cargo/Tauri environment and Git state; public builds never load bundle secrets.
 * [Output] Compile-time desktop build identity with the cross-end protocol schema.
 * [Pos] Tauri build-script boundary for release metadata and bundled resources.
 * [Sync] If the protocol schema changes, update the P4 CMake contract and tests.
 */

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PET_MANAGER_PROTOCOL_SCHEMA: u32 = 7;

fn git_output(repo_root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
}

fn emit_build_identity() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap_or_else(|| ".".into()));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or(&manifest_dir);
    let git_sha = git_output(repo_root, &["rev-parse", "--short=12", "HEAD"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = git_output(
        repo_root,
        &["status", "--porcelain", "--untracked-files=normal"],
    )
    .is_some_and(|value| !value.is_empty());
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let build_id = format!("{version}+{git_sha}{}", if dirty { "-dirty" } else { "" });

    println!(
        "cargo:rerun-if-changed={}",
        repo_root.join(".git/HEAD").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo_root.join(".git/index").display()
    );
    println!("cargo:rustc-env=PET_MANAGER_BUILD_GIT_SHA={git_sha}");
    println!(
        "cargo:rustc-env=PET_MANAGER_BUILD_DIRTY={}",
        u8::from(dirty)
    );
    println!("cargo:rustc-env=PET_MANAGER_BUILD_ID={build_id}");
    println!("cargo:rustc-env=PET_MANAGER_PROTOCOL_SCHEMA={PET_MANAGER_PROTOCOL_SCHEMA}");
}

fn main() {
    prepare_reference_image_bridge();
    emit_build_identity();
    if windows_msvc_target() {
        // Tauri embeds its manifest only into bin targets. `cargo test --lib`
        // links a separate executable, and `cargo:rustc-link-arg-tests` does not
        // reach that executable. Drop Tauri's bin-only copy and pass one manifest
        // with an unscoped linker argument so the app exe and the lib test exe
        // both embed it.
        let attributes = tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new_without_app_manifest(),
        );
        if let Err(error) = tauri_build::try_build(attributes) {
            println!("{error:#}");
            std::process::exit(1);
        }
        embed_windows_comctl32_v6_manifest();
    } else {
        tauri_build::build();
    }
}

fn windows_msvc_target() -> bool {
    env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
}

fn embed_windows_comctl32_v6_manifest() {
    let architecture = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "amd64",
        Ok("aarch64") => "arm64",
        Ok("x86") => "x86",
        other => panic!("unsupported Windows architecture for the comctl32 6.0 manifest: {other:?}"),
    };
    let manifest = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="{architecture}"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#
    );
    let manifest_path = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set for build scripts"))
        .join("comctl32-v6.xml");
    fs::write(&manifest_path, manifest).expect("failed to write the comctl32 6.0 manifest");
    let manifest_path = manifest_path
        .to_str()
        .expect("comctl32 manifest path is valid Unicode");
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{manifest_path}");
}

fn prepare_reference_image_bridge() {
    println!("cargo:rerun-if-changed=src/reference_image_macos.m");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") { return; }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let arch = if env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("aarch64") { "arm64" } else { "x86_64" };
    let object = out.join("reference_image.o");
    let status = Command::new("xcrun").args(["clang", "-fobjc-arc", "-fblocks", "-O2", "-arch", arch,
        "-mmacosx-version-min=10.15", "-Werror=implicit-function-declaration", "-c", "src/reference_image_macos.m", "-o"])
        .arg(&object).status().expect("Xcode command line tools are required for photo import");
    assert!(status.success(), "Photo import bridge failed to compile");
    let status = Command::new("xcrun").args(["ar", "crs"]).arg(out.join("libreference_image.a")).arg(object)
        .status().expect("Cannot archive photo import bridge");
    assert!(status.success());
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=reference_image");
    println!("cargo:rustc-link-arg=-Wl,-weak_framework,UniformTypeIdentifiers");
    for framework in ["AppKit", "PhotosUI", "ImageIO", "Foundation", "CoreGraphics"] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
}

