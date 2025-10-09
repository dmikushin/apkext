use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "apkext",
    about = "APK extraction and building tool with embedded JAR utilities",
    long_about = "APK extraction and building tool with embedded JAR utilities.\nSupports unpacking APK files to source code and repacking them back.",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Unpack APK file to source code
    #[command(
        about = "Unpack APK file to source code",
        long_about = "Unpack APK file by extracting resources, converting DEX to JAR,\nand decompiling Java classes to source code."
    )]
    Unpack {
        /// Path to the APK file to unpack
        #[arg(value_name = "APK_FILE")]
        apk_file: String,
    },

    /// Pack source code back to APK
    #[command(
        about = "Pack source code back to APK",
        long_about = "Pack the unpacked source code directory back into an APK file."
    )]
    Pack {
        /// Path to the unpacked directory
        #[arg(value_name = "UNPACKED_DIR")]
        unpacked_dir: String,

        /// Output APK file path
        #[arg(value_name = "OUTPUT_APK")]
        output_apk: String,
    },

    /// Start MCP (Model Context Protocol) server
    #[command(
        about = "Start MCP (Model Context Protocol) server",
        long_about = "Start MCP server to enable Claude and other AI assistants to use apkext tools.\nThis allows AI assistants to unpack and pack APK files through the Model Context Protocol."
    )]
    Mcp,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}