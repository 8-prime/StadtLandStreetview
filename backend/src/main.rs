use std::{env, error::Error, io};

use backend::{
    api::{self, ApiConfig, AppState},
    session::SessionManager,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = env::var("SLS_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let public_origin = env::var("SLS_PUBLIC_ORIGIN")
        .ok()
        .map(|origin| origin.parse::<axum::http::Uri>())
        .transpose()?;
    if public_origin.as_ref().is_some_and(|origin| {
        !matches!(origin.scheme_str(), Some("http" | "https"))
            || origin.authority().is_none()
            || origin.query().is_some()
            || origin.path() != "/" && !origin.path().is_empty()
    }) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "SLS_PUBLIC_ORIGIN must be an HTTP(S) origin without a path or query.",
        )
        .into());
    }
    let secure_cookies = match env::var("SLS_SECURE_COOKIES").as_deref() {
        Ok("true" | "1") => true,
        Ok("false" | "0") | Err(_) => false,
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "SLS_SECURE_COOKIES must be true or false.",
            )
            .into());
        }
    };
    let state = AppState::new(
        SessionManager::new(),
        ApiConfig {
            secure_cookies,
            public_origin,
        },
    );
    let app = api::router(state.clone());
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "StadtLandStreetview listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            if let Err(error) = tokio::signal::ctrl_c().await {
                eprintln!("Could not listen for Ctrl+C: {error}");
            }
            state.shutdown();
        })
        .await?;
    Ok(())
}
