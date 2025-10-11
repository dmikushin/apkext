use crate::Result;
use include_dir::{include_dir, Dir};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Embedded JAR files - downloaded during build.rs
static APKTOOL_JAR: &[u8] = include_bytes!("../../assets/jars/apktool.jar");
static PROCYON_JAR: &[u8] = include_bytes!("../../assets/jars/procyon-decompiler-v0.6.1.jar");

// Embedded tools directory
static TOOLS_DIR: Dir = include_dir!("assets/tools");

pub struct AssetManager {
    temp_dir: TempDir,
    tools_path: PathBuf,
}

impl AssetManager {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let tools_path = temp_dir.path().to_path_buf();

        let manager = Self {
            temp_dir,
            tools_path,
        };

        manager.extract_all()?;
        Ok(manager)
    }

    fn extract_all(&self) -> Result<()> {
        self.extract_jars()?;
        self.extract_tools()?;
        Ok(())
    }

    fn extract_jars(&self) -> Result<()> {
        let jars_dir = self.tools_path.join("jars");
        std::fs::create_dir_all(&jars_dir)?;

        // Extract JAR files
        std::fs::write(jars_dir.join("apktool.jar"), APKTOOL_JAR)?;
        std::fs::write(jars_dir.join("procyon-decompiler-v0.6.1.jar"), PROCYON_JAR)?;

        Ok(())
    }

    fn extract_tools(&self) -> Result<()> {
        self.extract_embedded_dir(&TOOLS_DIR, &self.tools_path)?;
        Ok(())
    }

    fn extract_embedded_dir(&self, embedded_dir: &Dir, dest_path: &Path) -> Result<()> {
        // Create the destination directory
        std::fs::create_dir_all(dest_path)?;

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
                std::fs::create_dir_all(parent)?;
            }

            // Write the file content
            std::fs::write(&dest_file_path, file.contents())?;

            // Make .sh files executable on Unix systems
            #[cfg(unix)]
            {
                if dest_file_path.extension().and_then(|s| s.to_str()) == Some("sh") {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(&dest_file_path)?.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&dest_file_path, perms)?;
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