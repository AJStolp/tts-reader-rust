use reqwest::Client;
use serde_json::json;
use std::env;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};

// Define the PollyClient struct
pub struct PollyClient {
    client: Client,
    endpoint: String,
    access_key: String,
    secret_key: String,
}

impl PollyClient {
    // Constructor for PollyClient
    pub fn new() -> Self {
      let endpoint = format!(
            "https://polly.{}.amazonaws.com",
            env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()) // Default to us-east-1
      );


        Self {
            client: Client::new(),
            endpoint,
            access_key: env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID not set"),
            secret_key: env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY not set"),
        }
    }

    // Generate AWS Signature Version 4
    fn generate_aws_signature(&self, payload: &str, host: &str, region: &str) -> String {
        // Get the current date and time in UTC
        let now = Utc::now();
        let date = now.format("%Y%m%d").to_string();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();

        // Step 1: Create the Canonical Request
        let canonical_uri = "/v1/speech";
        let canonical_query_string = "";
        let canonical_headers = format!("host:{}\nx-amz-date:{}\n", host, amz_date);
        let signed_headers = "host;x-amz-date";
        let payload_hash = hex::encode(Sha256::digest(payload.as_bytes()));
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            "POST",
            canonical_uri,
            canonical_query_string,
            canonical_headers,
            signed_headers,
            payload_hash
        );

        // Step 2: Create the String to Sign
        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/polly/aws4_request", date, region);
        let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            amz_date,
            credential_scope,
            canonical_request_hash
        );

        // Step 3: Calculate the Signature
        let k_date = sign(format!("AWS4{}", self.secret_key).as_bytes(), date.as_bytes());
        let k_region = sign(&k_date, region.as_bytes());
        let k_service = sign(&k_region, b"polly");
        let k_signing = sign(&k_service, b"aws4_request");
        let signature = hex::encode(sign(&k_signing, string_to_sign.as_bytes()));

        // Step 4: Build the Authorization Header
        format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm,
            self.access_key,
            credential_scope,
            signed_headers,
            signature
        )
    }

    // Send a TTS request to Amazon Polly
    pub async fn synthesize_speech(&self, text: &str, voice_id: &str) -> Result<String, reqwest::Error> {
        let payload = json!({
            "Text": text,
            "VoiceId": voice_id,
            "OutputFormat": "mp3"
        })
        .to_string();

        // Generate AWS Signature
        let signature = self.generate_aws_signature(
            &payload,
            "polly.us-west-2.amazonaws.com", // Update to your AWS region
            "us-west-2",                    // Update to your AWS region
        );

        // Make the HTTP POST request
        let response = self
            .client
            .post(&format!("{}/v1/speech", self.endpoint))
            .header("Authorization", signature)
            .header("x-amz-date", Utc::now().format("%Y%m%dT%H%M%SZ").to_string())
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await?;

        // Return the response as a string
        Ok(response.text().await?)
    }
}

// Helper function for HMAC signing
fn sign(key: &[u8], message: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(message);
    mac.finalize().into_bytes().to_vec()
}
