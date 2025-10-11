use crate::{config::Config, tools::ToolManager, assets::AssetManager, Result};
use std::path::Path;

pub struct Builder {
    tool_manager: ToolManager,
    asset_manager: AssetManager,
}

impl Builder {
    pub fn new(config: Config) -> Self {
        let tool_manager = ToolManager::new(config);
        let asset_manager = AssetManager::new().expect("Failed to create asset manager");
        Self { tool_manager, asset_manager }
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

        // Get platform-specific aapt path through asset manager
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

        let relative_path = match (env::consts::OS, env::consts::ARCH) {
            ("linux", "x86_64") => "prebuilt/prebuilt/linux/prebuilt/linux/aapt_64",
            ("linux", _) => "prebuilt/prebuilt/linux/prebuilt/linux/aapt",
            ("macos", _) => "prebuilt/prebuilt/macosx/prebuilt/macosx/aapt_64",
            ("windows", "x86_64") => "prebuilt/prebuilt/windows/prebuilt/windows/aapt_64.exe",
            ("windows", _) => "prebuilt/prebuilt/windows/prebuilt/windows/aapt.exe",
            _ => "prebuilt/prebuilt/linux/prebuilt/linux/aapt_64", // default
        };

        self.asset_manager.get_script_path(relative_path).to_string_lossy().to_string()
    }
}