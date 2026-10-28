use std::{env, error::Error, io, path::PathBuf, process::Command};

fn main() {
    if let Err(error) = build_frontend() {
        panic!("Could not build the embedded frontend: {error}");
    }
}

fn build_frontend() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let frontend = manifest.join("../frontend").canonicalize()?;
    for input in [
        "src",
        "public",
        "index.html",
        "package.json",
        "pnpm-lock.yaml",
        "vite.config.ts",
        "svelte.config.js",
        "tsconfig.json",
        ".env",
        ".env.local",
        ".env.production",
        ".env.production.local",
    ] {
        println!("cargo::rerun-if-changed={}", frontend.join(input).display());
    }
    println!("cargo::rerun-if-env-changed=VITE_GOOGLE_MAPS_API_KEY");

    let vite = frontend.join("node_modules/vite/bin/vite.js");
    if !vite.is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound,
            "Frontend dependencies are missing. Run `pnpm --dir frontend install --frozen-lockfile` from the repository root, then retry Cargo."
        ).into());
    }

    // Keep generated files inside Cargo's output directory, separate from Vite dev builds.
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build_root = output.canonicalize()?;
    let assets = output.join("frontend");
    // Vite will empty this directory. Verify an existing path does not escape OUT_DIR.
    if assets.exists() && assets.canonicalize()?.parent() != Some(build_root.as_path()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The frontend output directory must be a direct child of Cargo's OUT_DIR.",
        )
        .into());
    }
    let result = Command::new("node")
        .arg(vite)
        .arg("build")
        .arg("--outDir")
        .arg(&assets)
        .arg("--emptyOutDir")
        .env("VITE_API_MODE", "http")
        .env("NODE_ENV", "production")
        .current_dir(&frontend)
        .output()
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Could not run Node.js (install Node 22.12+): {error}"),
            )
        })?;
    if !result.status.success() {
        return Err(io::Error::other(format!(
            "Vite failed ({}):\n{}\n{}",
            result.status,
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr),
        ))
        .into());
    }
    if !assets.join("index.html").is_file() {
        return Err(
            io::Error::new(io::ErrorKind::NotFound, "Vite did not produce index.html.").into(),
        );
    }
    Ok(())
}
