use axum::{
    routing::{post, get_service},
    Router, Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any}; 
use tracing_subscriber;

mod polly;
use polly::PollyClient;

#[tokio::main]
async fn main() {
    // Initialize structured logging
    tracing_subscriber::fmt().init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Add CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow all origins
        .allow_methods(Any) // Allow all HTTP methods
        .allow_headers(Any); // Allow all headers

    // Define the application routes
    let app = Router::new()
        .route("/generate-tts", post(generate_tts))
        .route("/voices", post(get_voices))
        .nest_service("/", get_service(ServeDir::new("./")).handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error") }))
        .layer(cors); // Add the CORS layer here

    // Start the HTTP server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Request payload for TTS
#[derive(Deserialize, Debug)]
struct TtsRequest {
    text: String,
    voice_id: String,
}

// Response payload for TTS
// #[derive(Serialize)]
// struct TtsResponse {
//     audio_url: String,
// }

// Handler for generating TTS
async fn generate_tts(Json(payload): Json<TtsRequest>) -> Result<(StatusCode, Vec<u8>), (StatusCode, String)> {
    let polly_client = PollyClient::new();
    tracing::info!("Request received with payload: {:?}", payload);

    match polly_client.synthesize_speech(&payload.text, &payload.voice_id).await {
        Ok(audio_data) => {
            tracing::info!("TTS generation succeeded");
            Ok((
                StatusCode::OK,
                audio_data,
            ))
        }
        Err(e) => {
            tracing::error!("TTS generation failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}


// Placeholder for voices endpoint (to be implemented)
async fn get_voices() -> &'static str {
    "List of voices"
}
