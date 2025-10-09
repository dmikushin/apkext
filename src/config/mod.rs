use crate::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub java: JavaConfig,
    pub tools: ToolsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaConfig {
    pub java_path: String,
    pub java_home: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub apktool_jar: String,
    pub procyon_jar: String,
    pub aapt_path: String,
    pub dex2jar_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::load()
    }
}

impl Config {
    pub fn load() -> Self {
        let java = JavaConfig::detect();
        let tools = ToolsConfig::default();

        Self { java, tools }
    }

    pub fn update_tool_paths(&mut self, tools_dir: &std::path::Path) {
        self.tools.apktool_jar = tools_dir.join("jars/apktool.jar").to_string_lossy().to_string();
        self.tools.procyon_jar = tools_dir.join("jars/procyon-decompiler-v0.6.1.jar").to_string_lossy().to_string();

        // Platform-specific aapt paths
        self.tools.aapt_path = self.get_aapt_path(tools_dir);
        self.tools.dex2jar_path = self.get_dex2jar_path(tools_dir);
    }

    fn get_aapt_path(&self, tools_dir: &std::path::Path) -> String {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        let path = match (os, arch) {
            ("linux", "x86_64") => "prebuilt/linux/aapt_64",
            ("linux", _) => "prebuilt/linux/aapt",
            ("macos", _) => "prebuilt/macosx/aapt_64",
            ("windows", "x86_64") => "prebuilt/windows/aapt_64.exe",
            ("windows", _) => "prebuilt/windows/aapt.exe",
            _ => "prebuilt/linux/aapt", // fallback
        };

        tools_dir.join(path).to_string_lossy().to_string()
    }

    fn get_dex2jar_path(&self, tools_dir: &std::path::Path) -> String {
        let script = if cfg!(windows) {
            "dex-tools-v2.4/d2j-dex2jar.bat"
        } else {
            "dex-tools-v2.4/d2j-dex2jar.sh"
        };

        tools_dir.join(script).to_string_lossy().to_string()
    }
}

impl JavaConfig {
    pub fn detect() -> Self {
        let java_path = Self::find_java().unwrap_or_else(|| "java".to_string());
        let java_home = env::var("JAVA_HOME").ok();

        Self { java_path, java_home }
    }

    fn find_java() -> Option<String> {
        // Try JAVA_HOME first
        if let Ok(java_home) = env::var("JAVA_HOME") {
            let java_exe = if cfg!(windows) { "java.exe" } else { "java" };
            let java_path = std::path::Path::new(&java_home).join("bin").join(java_exe);
            if java_path.exists() {
                return Some(java_path.to_string_lossy().to_string());
            }
        }

        // Try PATH
        if let Ok(_) = std::process::Command::new("java").arg("-version").output() {
            return Some("java".to_string());
        }

        None
    }

    pub fn check_java(&self) -> Result<()> {
        let output = std::process::Command::new(&self.java_path)
            .arg("-version")
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run java: {}", e))?;

        if !output.status.success() {
            anyhow::bail!("Java is not working properly");
        }

        Ok(())
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            apktool_jar: "apktool.jar".to_string(),
            procyon_jar: "procyon-decompiler-v0.6.1.jar".to_string(),
            aapt_path: "aapt".to_string(),
            dex2jar_path: "d2j-dex2jar.sh".to_string(),
        }
    }
}