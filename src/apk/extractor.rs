use crate::{config::Config, tools::ToolManager, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct Extractor {
    tool_manager: ToolManager,
}

impl Extractor {
    pub fn new(config: Config) -> Self {
        let tool_manager = ToolManager::new(config);
        Self { tool_manager }
    }

    pub async fn unpack(&self, apk_path: &str) -> Result<()> {
        let apk_path = Path::new(apk_path);

        // Validate input
        if apk_path.extension().and_then(|s| s.to_str()) != Some("apk") {
            return Err(anyhow::anyhow!("File must have .apk extension"));
        }

        if !apk_path.exists() {
            return Err(anyhow::anyhow!("APK file does not exist: {}", apk_path.display()));
        }

        let extract_dir = self.get_extract_dir(apk_path);
        if extract_dir.exists() {
            println!("[+] Removing existing directory '{}'", extract_dir.display());
            fs::remove_dir_all(&extract_dir).await
                .map_err(|e| anyhow::anyhow!("Failed to remove existing directory: {}", e))?;
        }

        println!("[+] Extracting under '{}'", extract_dir.display());

        // Step 1: Extract resources using apktool
        self.extract_resources(apk_path, &extract_dir).await?;

        // Step 2: Extract classes.dex
        self.extract_dex(apk_path, &extract_dir).await?;

        // Step 3: Convert DEX to JAR
        self.convert_dex_to_jar(&extract_dir).await?;

        // Step 4: Decompile JAR to source
        self.decompile_jar(&extract_dir).await?;

        println!("");
        println!("[+] Resources and smali are in '{}/unpacked'", extract_dir.display());
        println!("[+] Decompiled classes in '{}/src'", extract_dir.display());

        Ok(())
    }

    fn get_extract_dir(&self, apk_path: &Path) -> PathBuf {
        let mut extract_dir = apk_path.to_path_buf();
        extract_dir.set_extension("");
        extract_dir
    }

    async fn extract_resources(&self, apk_path: &Path, extract_dir: &Path) -> Result<()> {
        println!("[+] Extracting resources");

        let unpacked_dir = extract_dir.join("unpacked");
        self.tool_manager.run_apktool(&["d", "-f", &apk_path.to_string_lossy(), "-o", &unpacked_dir.to_string_lossy()]).await
    }

    async fn extract_dex(&self, apk_path: &Path, extract_dir: &Path) -> Result<()> {
        println!("[+] Extracting classes.dex");

        // Try to extract classes.dex first
        let classes_dex_path = extract_dir.join("classes.dex");
        if let Err(_) = self.unzip_file(apk_path, extract_dir, "classes.dex").await {
            // If classes.dex doesn't exist, try class.dex and rename it
            if let Err(e) = self.unzip_file(apk_path, extract_dir, "class.dex").await {
                return Err(anyhow::anyhow!("Failed to extract DEX file: {}", e));
            }

            // Rename class.dex to classes.dex
            let old_path = extract_dir.join("class.dex");
            fs::rename(&old_path, &classes_dex_path).await
                .map_err(|e| anyhow::anyhow!("Failed to rename class.dex to classes.dex: {}", e))?;
        }

        Ok(())
    }

    async fn convert_dex_to_jar(&self, extract_dir: &Path) -> Result<()> {
        println!("[+] Converting classes.dex to jar");

        let dex_path = extract_dir.join("classes.dex");
        let jar_path = extract_dir.join("classes.jar");

        self.tool_manager.run_dex2jar(&[&dex_path.to_string_lossy(), "-o", &jar_path.to_string_lossy()]).await?;

        // Remove the DEX file after conversion
        fs::remove_file(&dex_path).await
            .map_err(|e| anyhow::anyhow!("Failed to remove DEX file: {}", e))?;

        Ok(())
    }

    async fn decompile_jar(&self, extract_dir: &Path) -> Result<()> {
        println!("[+] Decompiling jar files");

        let src_dir = extract_dir.join("src");
        let jar_path = extract_dir.join("classes.jar");

        // Remove existing src directory
        if src_dir.exists() {
            fs::remove_dir_all(&src_dir).await
                .map_err(|e| anyhow::anyhow!("Failed to remove existing src directory: {}", e))?;
        }

        // Create src directory
        fs::create_dir_all(&src_dir).await
            .map_err(|e| anyhow::anyhow!("Failed to create src directory: {}", e))?;

        // Run Procyon decompiler
        self.tool_manager.run_procyon(&["-jar", &jar_path.to_string_lossy(), "-o", &src_dir.to_string_lossy()]).await
    }

    async fn unzip_file(&self, zip_path: &Path, extract_dir: &Path, file_name: &str) -> Result<()> {
        use std::fs::File;
        use zip::ZipArchive;

        let file = File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name() == file_name {
                let outpath = extract_dir.join(file_name);

                if let Some(p) = outpath.parent() {
                    std::fs::create_dir_all(p)?;
                }

                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("File {} not found in archive", file_name))
    }
}