use axum::{
    routing::{get_service, post},
    Router, Json,
    body::StreamBody,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber;
use tokio_util::io::StreamReader;
use futures::TryStreamExt;
use tokio_util::io::ReaderStream;

mod polly;
use polly::PollyClient;

#[tokio::main]
async fn main() {
    // Initialize structured logging
    tracing_subscriber::fmt().init();

    // Load environment variables
    dotenv::dotenv().ok();

    // CORS settings
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Define routes
    let app = Router::new()
        .route("/generate-tts", post(generate_tts))
        .route("/voices", post(get_voices))
        .nest_service("/", get_service(ServeDir::new("./")).handle_error(|_| async {
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        }))
        .layer(cors);

    // Start server
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

async fn generate_tts(
    Json(payload): Json<TtsRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let polly_client = PollyClient::new();
    match polly_client.synthesize_stream(&payload.text, &payload.voice_id).await {
        Ok(stream) => {
            // Map `reqwest::Error` to `std::io::Error` and ensure it satisfies TryStream
            let io_stream = stream.map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Reqwest Error: {}", e))
            });

            // Use `tokio_util::io::ReaderStream` directly to convert to a `Stream` stony
            let body = StreamBody::new(ReaderStream::new(StreamReader::new(io_stream)));

            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", HeaderValue::from_static("audio/mpeg"));

            Ok((headers, body))
        }
        Err(err) => {
            tracing::error!("Error generating TTS: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_voices() -> impl IntoResponse {
    Json(vec!["Joanna", "Matthew", "Amy", "Brian"])
}
