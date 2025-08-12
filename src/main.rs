mod tools;

use anyhow::Result;
use clap::Parser;
use rmcp::{
    model::*, service::ServerInitializeError, transport::stdio, ErrorData as McpError, ServiceExt,
};
use tokio::io::AsyncWriteExt;
use tools::NuServer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to custom nushell config.nu file
    #[arg(long = "nu-config")]
    nu_config: Option<String>,

    /// Path to custom nushell env.nu file
    #[arg(long = "nu-env-config")]
    nu_env_config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create and start the Nushell MCP server
    loop {
        match NuServer::new(args.nu_config.clone(), args.nu_env_config.clone())
            .serve(stdio())
            .await
        {
            Ok(service) => {
                service.waiting().await?;
                break;
            }
            Err(ServerInitializeError::ExpectedInitializeRequest(Some(message))) => {
                // Extract request ID if possible and send proper error response
                if let Some((_, id)) = message.into_request() {
                    let error_response = ServerJsonRpcMessage::error(
                        McpError::invalid_request(
                            "Server not initialized. Please send initialize request first.",
                            Some(serde_json::json!({
                                "error": "Pre-initialization request received",
                                "required_flow": [
                                    "1. Send initialize request",
                                    "2. Wait for initialize response",
                                    "3. Send initialized notification",
                                    "4. Then other requests are allowed"
                                ]
                            })),
                        ),
                        id,
                    );

                    let error_json = serde_json::to_string(&error_response)?;
                    let mut stdout = tokio::io::stdout();
                    stdout
                        .write_all(format!("{error_json}\n").as_bytes())
                        .await?;
                    stdout.flush().await?;
                }

                // Continue the loop to try serving again
                continue;
            }
            Err(e) => {
                // Let other initialization errors propagate normally
                return Err(anyhow::anyhow!("Server initialization failed: {}", e));
            }
        }
    }

    Ok(())
}
