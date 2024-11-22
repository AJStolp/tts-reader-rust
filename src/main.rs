use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::time::Duration;

use clipboard::{ClipboardContext, ClipboardProvider};
use dotenv::dotenv;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use rodio::{Decoder, OutputStream, Sink};
use serde::Serialize;

#[derive(Serialize)]
struct ElevenLabsPayload<'a> {
    text: &'a str,
}

fn get_clipboard_text() -> Result<String, Box<dyn std::error::Error>> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    let text = ctx.get_contents()?;
    if text.trim().is_empty() {
        return Err("Clipboard is empty".into());
    }
    Ok(text)
}

fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok(); // Load environment variables from .env
    let api_key = env::var("ELEVENLABS_API_KEY")?;
    Ok(api_key)
}

fn generate_speech(api_key: &str, text: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15)) // Set a 15-second timeout
        .build()?;

    let voice_id = "9BWtsMINqrJLrRacOk9x"; // Replace with a valid voice ID
    let endpoint = format!("https://api.elevenlabs.io/v1/text-to-speech/{}/stream", voice_id);

    let payload = ElevenLabsPayload { text };

    // Set headers as per the documentation
    let mut headers = HeaderMap::new();
    headers.insert("xi-api-key", HeaderValue::from_str(api_key)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("Accept", HeaderValue::from_static("audio/mp3")); // Request MP3 format

    // Make the request
    println!("Sending request to ElevenLabs API for text: {}", text);

    let response = client.post(&endpoint)
        .headers(headers)
        .json(&payload)
        .send()?;

    if response.status().is_success() {
        println!("Successfully received response from ElevenLabs API");
        let audio_bytes = response.bytes()?;
        println!("Audio bytes length: {}", audio_bytes.len());

        if audio_bytes.is_empty() {
            eprintln!("No audio data received.");
            return Err("No audio data received".into());
        }

        Ok(audio_bytes.to_vec())
    } else {
        let error_message = response.text()?;
        eprintln!("Failed to generate speech: {}", error_message);
        Err(format!("Failed to generate speech: {}", error_message).into())
    }
}

fn save_audio_to_file(audio_bytes: &[u8], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filename)?;
    file.write_all(audio_bytes)?;
    println!("Audio saved to file: {}", filename);
    Ok(())
}

fn play_audio_from_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Attempting to play audio from file...");

    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("Failed to create output stream: {}", e))?;

    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| format!("Failed to create sink: {}", e))?;

    let file = File::open(Path::new(filename)).map_err(|e| format!("Failed to open audio file: {}", e))?;
    let source = Decoder::new(BufReader::new(file))
        .map_err(|e| format!("Failed to decode audio: {}", e))?;

    println!("Audio successfully decoded. Starting playback...");

    sink.append(source);
    sink.sleep_until_end();

    println!("Finished playing audio.");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;

    // Keep attempting to get the clipboard text until we find valid content
    loop {
        let text = match get_clipboard_text() {
            Ok(txt) => txt,
            Err(_) => {
                println!("Clipboard is empty or failed to retrieve. Please copy some text and try again.");
                std::thread::sleep(Duration::from_secs(2)); // Check every 2 seconds
                continue;
            }
        };

        // If clipboard text is retrieved successfully, proceed
        if !text.trim().is_empty() {
            println!("New text found in clipboard. Generating speech...");

            // Generate Speech
            let audio_bytes = generate_speech(&api_key, &text)?;

            // Save the Generated Speech to a File
            save_audio_to_file(&audio_bytes, "output.mp3")?;

            // Play the Generated Speech from file
            play_audio_from_file("output.mp3")?;

            break; // Exit after successful processing
        }
    }

    Ok(())
}
