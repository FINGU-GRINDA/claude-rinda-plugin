use rmcp::{ServerHandler, ServiceExt, model::*, transport::stdio};

struct RindaMcpServer;

impl ServerHandler for RindaMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("rinda-mcp", env!("CARGO_PKG_VERSION")))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = RindaMcpServer;
    let transport = stdio();
    let service = server.serve(transport).await?;
    service.waiting().await?;
    Ok(())
}
