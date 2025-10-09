use apkext::cli::{Cli, Commands};
use apkext::{apk, assets, config, mcp, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();

    // Initialize asset manager and config
    let asset_manager = assets::AssetManager::new()?;
    let mut config = config::Config::load();
    config.update_tool_paths(asset_manager.get_tools_path());

    // Check Java installation
    config.java.check_java()?;

    match cli.command {
        Commands::Unpack { apk_file } => {
            let extractor = apk::Extractor::new(config);
            extractor.unpack(&apk_file).await?;
        }

        Commands::Pack {
            unpacked_dir,
            output_apk,
        } => {
            let builder = apk::Builder::new(config);
            builder.pack(&unpacked_dir, &output_apk).await?;
        }

        Commands::Mcp => {
            let server = mcp::Server::new(config, asset_manager)?;
            server.run().await?;
        }
    }

    Ok(())
}
