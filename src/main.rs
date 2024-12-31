use axum::{
    routing::{get_service, post},
    Router, Json,
    body::StreamBody,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber;
use futures::{Stream, TryStreamExt};
use bytes::Bytes;

mod polly;
use polly::PollyClient;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    dotenv::dotenv().ok();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/generate-tts", post(generate_tts))
        .route("/audio-stream", post(stream_audio))
        .route("/voices", post(get_voices))
        .nest_service("/", get_service(ServeDir::new("./")).handle_error(|_| async {
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        }))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct TtsRequest {
    text: String,
    voice_id: String,
}

#[derive(Debug, Serialize)]
struct TtsResponse {
    audio_url: String,
    metadata: Vec<serde_json::Value>,
}

/// Generate TTS response with audio URL and metadata
async fn generate_tts(
    Json(payload): Json<TtsRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let polly_client = PollyClient::new();

    // Fetch speech marks
    let speech_marks = polly_client
        .get_speech_marks(&payload.text, &payload.voice_id)
        .await
        .map_err(|err| {
            tracing::error!("Speech marks error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = TtsResponse {
        audio_url: "/audio-stream".to_string(), // Replace with actual storage path or dynamic endpoint
        metadata: speech_marks,
    };

    Ok(Json(response))
}

/// Stream audio from Polly
async fn stream_audio(
    Json(payload): Json<TtsRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let polly_client = PollyClient::new();

    let audio_stream = polly_client
        .synthesize_stream(&payload.text, &payload.voice_id)
        .await
        .map_err(|err| {
            tracing::error!("Audio stream error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Convert the stream to a format compatible with StreamBody
    let audio_stream = audio_stream.map_err(|err| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Stream error: {}", err))
    });

    let body = StreamBody::new(audio_stream);

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("audio/mpeg"));

    Ok((headers, body))
}

async fn get_voices() -> impl IntoResponse {
    Json(vec!["Joanna", "Matthew", "Amy", "Brian"])
}
