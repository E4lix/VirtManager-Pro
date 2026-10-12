use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod drivers;
mod models;
mod state;
mod vnc;

use api::create_router;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "virtmanager_backend=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_path = PathBuf::from("data/database.json");
    let state = Arc::new(AppState::new(db_path).await);

    // Setup CORS for Frontend React
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut app = create_router(state.clone()).layer(cors);

    // Serve noVNC static assets if available
    if std::path::Path::new("/usr/share/novnc").exists() {
        app = app.nest_service("/novnc", ServeDir::new("/usr/share/novnc"));
        info!("Serving noVNC static assets at /novnc");
    }

    // Serve Frontend build if dist/ exists
    let dist_path = PathBuf::from("../frontend/dist");
    if dist_path.exists() {
        app = app.fallback_service(ServeDir::new(dist_path));
        info!("Serving production frontend assets from ../frontend/dist");
    }

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!(r#"
========================================================================
   🚀 VirtManager Pro (Proxmox-like Cloud Management Platform)
   Stack: Rust (Backend Axum) + Node.js/React (Frontend)
========================================================================
   📡 API Server & WebSockets : http://127.0.0.1:{port}
   📖 Documentation Swagger UI: http://127.0.0.1:{port}/swagger-ui/
   🖥️ Console noVNC Interface  : http://127.0.0.1:{port}/novnc/vnc.html
   🔑 Comptes par défaut      : 
      - Admin : admin / admin
      - User  : user1 / user1
========================================================================
"#, port = port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Backend listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
