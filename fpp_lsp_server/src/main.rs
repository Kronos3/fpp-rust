mod analysis;
mod diagnostics;
mod dispatcher;
mod global_state;
mod handlers;
mod lsp_ext;
mod notification;
mod request;
mod util;

mod lsp;
mod vfs;

pub use vfs::*;

use crate::{global_state::GlobalState, util::from_json};
use lsp_server::Connection;
use std::error::Error;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

fn setup_stderr_logging() -> anyhow::Result<()> {
    let stderr_log_level = tracing_subscriber::filter::LevelFilter::INFO;
    let stderr_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);

    tracing_subscriber::registry()
        .with(
            stderr_layer
                .with_ansi(false)
                .without_time()
                .with_target(false)
                .with_file(true)
                .with_line_number(true)
                .with_filter(stderr_log_level),
        )
        .try_init()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    setup_stderr_logging()?;

    // transport
    let (connection, io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = match connection.initialize_start() {
        Ok(it) => it,
        Err(e) => {
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };

    tracing::info!("InitializeParams: {}", initialize_params);
    let lsp_types::InitializeParams {
        capabilities,
        // workspace_folders,
        // initialization_options,
        // client_info,
        ..
    } = from_json::<lsp_types::InitializeParams>("InitializeParams", &initialize_params)?;

    let client_capabilities = lsp::capabilities::ClientCapabilities::new(capabilities);
    let server_capabilities = lsp::capabilities::server_capabilities(&client_capabilities);

    let initialize_result = lsp_types::InitializeResult {
        capabilities: server_capabilities,
        server_info: Some(lsp_types::ServerInfo {
            name: String::from("fpp"),
            version: Some("1.0.0".to_string()),
        }),
    };

    let initialize_result = serde_json::to_value(initialize_result)?;

    if let Err(e) = connection.initialize_finish(initialize_id, initialize_result) {
        if e.channel_is_disconnected() {
            io_threads.join()?;
        }
        return Err(e.into());
    }

    tracing::info!("server is starting up");
    GlobalState::run(connection, client_capabilities);
    io_threads.join()?;
    log::info!("shutting down server");

    Ok(())
}
