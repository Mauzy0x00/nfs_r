/*
*
*   Purpose: Reimagine NFS with security in mind using a memory safe programming language, Rust. 
*               This will be re-build from the bottom up. Striving first for functionality with security by default, 
*               then focusing on user experience. 
*
*   Author: Mauzy0x00
*   Start Date: 03-29-24 
*
*   File Description: This is the main function of the program. Supporting functions and loops will be called here
*/

mod client;
mod server;
mod encryption;
mod filesystem;
mod filesystem_linux;
mod filesystem_windows;
mod async_io;
mod protocol;
mod error;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn on verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start NFS server
    Server {
        /// Directory to export
        #[arg(short, long)]
        export_path: PathBuf,

        /// Address to bind to
        #[arg(short, long, default_value = "0.0.0.0:2049")]
        bind_address: String,
    },
    /// Start NFS client and mount a remote filesystem
    Client {
        /// Server address
        #[arg(short, long)]
        server: String,

        /// Local mount point
        #[arg(short, long)]
        mount_point: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logger
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Generate encryption keypair
    let keypair = encryption::KeyPair::generate();

    match cli.command {
        Commands::Server { export_path, bind_address } => {
            log::info!("Starting NFS server, exporting {} on {}", export_path.display(), bind_address);

            // Create and run server
            let server = server::NfsServer::new(export_path, bind_address, keypair)?;

            async_std::task::block_on(server.run())?;
        },
        Commands::Client { server, mount_point } => {
            log::info!("Starting NFS client, connecting to {} and mounting at {}", server, mount_point.display());

            // Create and run client
            let mut client = client::NfsClient::new(server, mount_point, keypair);

            async_std::task::block_on(async {
                client.connect().await?;

                log::info!("Connected to server. Press Ctrl+C to disconnect.");
                
                // TODO:
                // Implement input loop for the client 
                client.run().await?;

                // // Create remote directory
                // let remote_dir_path = "remote_test_dir"; 
                // let mode: u32 = 0o755;
                // match client.create_directory(remote_dir_path, mode).await {
                //     Ok(_) => log::info!("Successfully created directory: {}", remote_dir_path),
                //     Err(e) => log::error!("Error creating directory: {}", e),
                // }

                // // Wait for Ctrl+C
                // let (tx, rx) = async_std::channel::bounded(1);
                // ctrlc::set_handler(move || {
                //     let _ = tx.try_send(());
                // })?;

                // let _ = rx.recv().await;

                client.disconnect().await?;
                log::info!("Disconnected from server");

                Ok::<(), error::NfsError>(())
            })?;
        },
    }

    Ok(())
}