use crate::Result;
use include_dir::{include_dir, Dir};
use std::path::{Path, PathBuf};
use std::fs;

// Embedded JAR files - downloaded during build.rs
static APKTOOL_JAR: &[u8] = include_bytes!("../../assets/jars/apktool.jar");
static PROCYON_JAR: &[u8] = include_bytes!("../../assets/jars/procyon-decompiler-v0.6.1.jar");

// Embedded tools directory
static TOOLS_DIR: Dir = include_dir!("assets/tools");

pub struct AssetManager {
    tools_path: PathBuf,
}

impl AssetManager {
    pub fn new() -> Result<Self> {
        let tools_path = Self::get_config_dir()?;

        let manager = Self { tools_path };

        // Only extract if assets don't exist or are outdated
        if manager.needs_extraction()? {
            manager.extract_all()?;
        }

        Ok(manager)
    }

    fn get_config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("apkext");

        fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    fn needs_extraction(&self) -> Result<bool> {
        // Check if all required files exist
        let jars_dir = self.tools_path.join("jars");
        let apktool_jar = jars_dir.join("apktool.jar");
        let procyon_jar = jars_dir.join("procyon-decompiler-v0.6.1.jar");

        // Check if version file exists and matches current version
        let version_file = self.tools_path.join(".version");
        let current_version = env!("CARGO_PKG_VERSION");

        if !apktool_jar.exists() || !procyon_jar.exists() || !version_file.exists() {
            return Ok(true);
        }

        // Check version
        match fs::read_to_string(&version_file) {
            Ok(stored_version) => Ok(stored_version.trim() != current_version),
            Err(_) => Ok(true),
        }
    }

    fn extract_all(&self) -> Result<()> {
        println!("Extracting assets to config directory...");
        self.extract_jars()?;
        self.extract_tools()?;
        self.write_version_file()?;
        println!("Assets extracted successfully.");
        Ok(())
    }

    fn write_version_file(&self) -> Result<()> {
        let version_file = self.tools_path.join(".version");
        let current_version = env!("CARGO_PKG_VERSION");
        fs::write(version_file, current_version)?;
        Ok(())
    }

    fn extract_jars(&self) -> Result<()> {
        let jars_dir = self.tools_path.join("jars");
        fs::create_dir_all(&jars_dir)?;

        // Extract JAR files
        fs::write(jars_dir.join("apktool.jar"), APKTOOL_JAR)?;
        fs::write(jars_dir.join("procyon-decompiler-v0.6.1.jar"), PROCYON_JAR)?;

        Ok(())
    }

    fn extract_tools(&self) -> Result<()> {
        self.extract_embedded_dir(&TOOLS_DIR, &self.tools_path)?;
        Ok(())
    }

    fn extract_embedded_dir(&self, embedded_dir: &Dir, dest_path: &Path) -> Result<()> {
        // Create the destination directory
        fs::create_dir_all(dest_path)?;

        // Recursively extract files from the embedded directory
        self.extract_dir_recursive(embedded_dir, dest_path, "")?;

        Ok(())
    }

    fn extract_dir_recursive(&self, embedded_dir: &Dir, dest_path: &Path, current_path: &str) -> Result<()> {
        // Extract all files in current directory
        for file in embedded_dir.files() {
            let file_path = if current_path.is_empty() {
                file.path().to_path_buf()
            } else {
                Path::new(current_path).join(file.path())
            };

            let dest_file_path = dest_path.join(&file_path);

            // Create parent directories if they don't exist
            if let Some(parent) = dest_file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Write the file content
            fs::write(&dest_file_path, file.contents())?;

            // Make .sh files executable on Unix systems
            #[cfg(unix)]
            {
                if dest_file_path.extension().and_then(|s| s.to_str()) == Some("sh") {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&dest_file_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&dest_file_path, perms)?;
                }
            }
        }

        // Recursively extract subdirectories
        for subdir in embedded_dir.dirs() {
            let subdir_path = if current_path.is_empty() {
                subdir.path().to_string_lossy().to_string()
            } else {
                format!("{}/{}", current_path, subdir.path().to_string_lossy())
            };

            self.extract_dir_recursive(subdir, dest_path, &subdir_path)?;
        }

        Ok(())
    }


    pub fn get_jar_path(&self, jar_name: &str) -> PathBuf {
        self.tools_path.join("jars").join(jar_name)
    }

    pub fn get_script_path(&self, script_path: &str) -> PathBuf {
        self.tools_path.join(script_path)
    }

    pub fn get_tools_path(&self) -> &Path {
        &self.tools_path
    }
}