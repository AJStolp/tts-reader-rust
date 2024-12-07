use axum::{routing::post, Router, Json};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing_subscriber;
mod polly;

use polly::PollyClient;

#[tokio::main]
async fn main() {
    // Initialize structured logging
    tracing_subscriber::fmt().init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Define the application routes
    let app = Router::new()
        .route("/generate-tts", post(generate_tts))
        .route("/voices", post(get_voices));

    // Start the HTTP server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Request payload for TTS
#[derive(Deserialize)]
struct TtsRequest {
    text: String,
    voice_id: String,
}

// Response payload for TTS
#[derive(Serialize)]
struct TtsResponse {
    audio_url: String,
}

// Handler for generating TTS
async fn generate_tts(Json(payload): Json<TtsRequest>) -> Json<TtsResponse> {
    let polly_client = PollyClient::new();
    match polly_client.synthesize_speech(&payload.text, &payload.voice_id).await {
        Ok(audio_url) => Json(TtsResponse { audio_url }),
        Err(_) => Json(TtsResponse {
            audio_url: "Error generating TTS".to_string(),
        }),
    }
}

// Placeholder for voices endpoint (to be implemented)
async fn get_voices() -> &'static str {
    "List of voices"
}
