use axum::{
    routing::post,
    Router, Json,
};
use axum::response::{IntoResponse, Response};
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;
use tracing_subscriber;
use serde::{Deserialize, Serialize}; // Import serde macros
mod polly;

use polly::PollyClient;

#[tokio::main]
async fn main() {
    // Initialize structured logging
    tracing_subscriber::fmt().init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Define the CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any) // Allows requests from any origin. Replace `Any` with specific origins for security.
        .allow_methods(Any)
        .allow_headers(Any);

    // Define the application routes
    let app = Router::new()
        .route("/generate-tts", post(generate_tts))
        .route("/voices", post(get_voices))
        .layer(cors);

    // Start the HTTP server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)] // Added Debug
struct TtsRequest {
    text: String,
    voice_id: String,
}

#[derive(Serialize)] // Serialize for response
struct TtsResponse {
    audio_url: String,
}

// Handler for generating TTS
async fn generate_tts(Json(payload): Json<TtsRequest>) -> Response {
    tracing::info!("Request received with payload: {:?}", payload);

    let polly_client = PollyClient::new();
    match polly_client.synthesize_speech(&payload.text, &payload.voice_id).await {
        Ok(audio_url) => {
            tracing::info!("TTS generation succeeded. Audio URL: {}", audio_url);
            Json(TtsResponse { audio_url }).into_response()
        }
        Err(e) => {
            tracing::error!("TTS generation failed: {:?}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Error generating TTS".to_string(),
            )
                .into_response()
        }
    }
}

// Placeholder for voices endpoint (to be implemented)
async fn get_voices() -> &'static str {
    "List of voices"
}
