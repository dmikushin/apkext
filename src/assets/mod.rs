use crate::Result;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Embedded JAR files - downloaded during build.rs
static APKTOOL_JAR: &[u8] = include_bytes!("../../assets/jars/apktool.jar");
static PROCYON_JAR: &[u8] = include_bytes!("../../assets/jars/procyon-decompiler-v0.6.1.jar");

// TODO: Embedded tools directory will be added later
// const TOOLS_DIR: include_dir::Dir = include_dir::include_dir!("assets/tools");

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
        // TODO: Extract embedded tools directory
        println!("Tools extraction will be implemented later");
        Ok(())
    }

    #[cfg(unix)]
    fn make_scripts_executable(&self) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let tools_path = &self.tools_path;
        for entry in walkdir::WalkDir::new(tools_path) {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("sh") {
                let mut perms = entry.metadata()?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(entry.path(), perms)?;
            }
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