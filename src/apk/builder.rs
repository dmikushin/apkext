use crate::{config::Config, tools::ToolManager, Result};
use std::path::Path;

pub struct Builder {
    tool_manager: ToolManager,
}

impl Builder {
    pub fn new(config: Config) -> Self {
        let tool_manager = ToolManager::new(config);
        Self { tool_manager }
    }

    pub async fn pack(&self, unpacked_dir: &str, output_apk: &str) -> Result<()> {
        let unpacked_dir = Path::new(unpacked_dir);
        let output_apk = Path::new(output_apk);

        // Validate input
        if !unpacked_dir.exists() {
            return Err(anyhow::anyhow!("Unpacked directory does not exist: {}", unpacked_dir.display()));
        }

        if output_apk.extension().and_then(|s| s.to_str()) != Some("apk") {
            return Err(anyhow::anyhow!("Output file must have .apk extension"));
        }

        println!("[+] Building APK from '{}' to '{}'", unpacked_dir.display(), output_apk.display());

        // Get platform-specific aapt path
        let aapt_path = self.get_aapt_path();

        // Run apktool to build APK
        let unpacked_path = unpacked_dir.join("unpacked");
        let final_unpacked_path = if unpacked_path.join("apktool.yml").exists() {
            unpacked_path
        } else if unpacked_dir.join("apktool.yml").exists() {
            unpacked_dir.to_path_buf()
        } else {
            return Err(anyhow::anyhow!("apktool.yml not found in {} or {}", unpacked_dir.display(), unpacked_path.display()));
        };

        self.tool_manager.run_apktool(&[
            "b",
            "-aapt", &aapt_path,
            &final_unpacked_path.to_string_lossy(),
            "-o", &output_apk.to_string_lossy()
        ]).await
    }

    fn get_aapt_path(&self) -> String {
        use std::env;

        let base_path = match (env::consts::OS, env::consts::ARCH) {
            ("linux", "x86_64") => "prebuilt/linux/aapt_64",
            ("linux", _) => "prebuilt/linux/aapt",
            ("macos", _) => "prebuilt/macosx/aapt_64",
            ("windows", "x86_64") => "prebuilt/windows/aapt_64.exe",
            ("windows", _) => "prebuilt/windows/aapt.exe",
            _ => "prebuilt/linux/aapt_64", // default
        };

        base_path.to_string()
    }
}