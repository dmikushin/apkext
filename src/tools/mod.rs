use crate::{assets::AssetManager, config::Config, Result};
use tokio::process::Command;

pub struct ToolManager {
    config: Config,
    asset_manager: AssetManager,
}

impl ToolManager {
    pub fn new(config: Config) -> Self {
        let asset_manager = AssetManager::new().expect("Failed to create asset manager");
        Self { config, asset_manager }
    }

    pub async fn run_apktool(&self, args: &[&str]) -> Result<()> {
        let jar_path = self.asset_manager.get_jar_path("apktool.jar");
        let framework_path = self.asset_manager.get_tools_path().join("framework");

        // Ensure framework directory exists
        tokio::fs::create_dir_all(&framework_path).await?;

        // Build command: java -jar apktool.jar [command] --frame-path <path> [other args]
        let jar_path_str = jar_path.to_string_lossy().to_string();
        let framework_path_str = framework_path.to_string_lossy().to_string();

        let mut cmd_args = vec!["-jar", &jar_path_str];

        // If there are args and the first one is a command (d, b, etc), add it before frame-path
        if !args.is_empty() {
            cmd_args.push(args[0]);
            cmd_args.push("--frame-path");
            cmd_args.push(&framework_path_str);
            cmd_args.extend_from_slice(&args[1..]);
        } else {
            cmd_args.push("--frame-path");
            cmd_args.push(&framework_path_str);
            cmd_args.extend_from_slice(args);
        }

        let mut cmd = Command::new(&self.config.java.java_path);
        cmd.args(&cmd_args);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Apktool failed: {}", stderr));
        }

        // Print stdout for user feedback
        print!("{}", String::from_utf8_lossy(&output.stdout));

        Ok(())
    }

    pub async fn run_procyon(&self, args: &[&str]) -> Result<()> {
        let jar_path = self.asset_manager.get_jar_path("procyon-decompiler-v0.6.1.jar");
        let jar_path_str = jar_path.to_string_lossy().to_string();

        let mut cmd_args = vec!["-jar", &jar_path_str];
        cmd_args.extend_from_slice(args);

        let mut cmd = Command::new(&self.config.java.java_path);
        cmd.args(&cmd_args);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Procyon failed: {}", stderr));
        }

        // Print stdout for user feedback
        print!("{}", String::from_utf8_lossy(&output.stdout));

        Ok(())
    }

    pub async fn run_dex2jar(&self, args: &[&str]) -> Result<()> {
        let script_path = self.asset_manager.get_script_path("dex-tools-v2.4/d2j-dex2jar.sh");

        let mut cmd = Command::new("bash");
        cmd.arg(&script_path);
        cmd.args(args);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Dex2jar failed: {}", stderr));
        }

        // Print stdout for user feedback
        print!("{}", String::from_utf8_lossy(&output.stdout));

        Ok(())
    }
}