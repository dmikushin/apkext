use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    println!("cargo:rerun-if-changed=build.rs");

    // Create directories for embedded assets
    let jars_dir = Path::new("assets/jars");
    let tools_dir = Path::new("assets/tools");

    fs::create_dir_all(&jars_dir).context("Failed to create jars directory")?;
    fs::create_dir_all(&tools_dir).context("Failed to create tools directory")?;

    println!("cargo:warning=Downloading dependencies during build...");

    // Download JAR files
    rt.block_on(download_jar_files(&jars_dir))?;

    // Download and extract dex2jar tools
    rt.block_on(download_dex2jar(&tools_dir))?;

    // Extract aapt from apktool
    rt.block_on(extract_aapt_from_apktool(&jars_dir, &tools_dir))?;

    println!("cargo:warning=All dependencies downloaded successfully!");
    Ok(())
}

async fn download_jar_files(jars_dir: &Path) -> Result<()> {
    let downloads = vec![
        (
            "apktool.jar",
            "https://github.com/iBotPeaches/Apktool/releases/download/v2.12.1/apktool_2.12.1.jar"
        ),
        (
            "procyon-decompiler-v0.6.1.jar",
            "https://github.com/dmikushin/procyon/releases/download/v0.6.1/procyon-decompiler-v0.6.1.jar"
        ),
    ];

    for (filename, url) in downloads {
        let file_path = jars_dir.join(filename);

        if file_path.exists() {
            println!("cargo:warning={} already exists, skipping", filename);
            continue;
        }

        println!("cargo:warning=Downloading {}...", filename);
        download_file(url, &file_path).await
            .with_context(|| format!("Failed to download {}", filename))?;
    }

    Ok(())
}

async fn download_dex2jar(tools_dir: &Path) -> Result<()> {
    let dex2jar_url = "https://github.com/pxb1988/dex2jar/releases/download/v2.4/dex-tools-v2.4.zip";
    let zip_path = tools_dir.join("dex-tools.zip");

    if tools_dir.join("dex-tools-v2.4").exists() {
        println!("cargo:warning=dex2jar already exists, skipping");
        return Ok(());
    }

    println!("cargo:warning=Downloading dex2jar...");
    download_file(dex2jar_url, &zip_path).await?;

    println!("cargo:warning=Extracting dex2jar...");
    extract_zip(&zip_path, tools_dir)?;

    // Clean up zip file
    let _ = fs::remove_file(&zip_path);

    Ok(())
}

async fn extract_aapt_from_apktool(jars_dir: &Path, tools_dir: &Path) -> Result<()> {
    let apktool_path = jars_dir.join("apktool.jar");
    let prebuilt_dir = tools_dir.join("prebuilt");

    if prebuilt_dir.exists() {
        println!("cargo:warning=aapt already extracted, skipping");
        return Ok(());
    }

    println!("cargo:warning=Extracting aapt from apktool...");
    extract_from_jar(&apktool_path, "prebuilt/*", tools_dir)?;

    Ok(())
}

async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }

    let bytes = response.bytes().await?;
    fs::write(dest, bytes)?;

    Ok(())
}

fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;

            // Make scripts executable on Unix
            #[cfg(unix)]
            {
                if outpath.extension().and_then(|s| s.to_str()) == Some("sh") {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = outfile.metadata()?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&outpath, perms)?;
                }
            }
        }
    }

    Ok(())
}

fn extract_from_jar(jar_path: &Path, pattern: &str, dest_dir: &Path) -> Result<()> {
    let file = fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name();

        // Simple pattern matching for "prebuilt/*"
        if pattern == "prebuilt/*" && !file_name.starts_with("prebuilt/") {
            continue;
        }

        let outpath = dest_dir.join(file_name);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}