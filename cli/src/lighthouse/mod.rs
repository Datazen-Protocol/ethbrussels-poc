use dotenv::dotenv;
use reqwest::multipart;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Write;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
#[derive(Deserialize, Debug)]
pub struct LighthouseResponse {
    Name: String,
    Hash: String,
    Size: String,
}

pub async fn upload_file(base64_file_path: &str) -> Result<LighthouseResponse, Box<dyn Error>> {
    log::info!("Backing Up file on Filecoin!! Your encrypted data isn't going anywhere now 😈");
    let mut file = File::open(base64_file_path)?;

    dotenv().ok();
    let lh_key = std::env::var("LH_API").expect("LH_API_TOKEN must be set.");
    const SIZE_LIMIT: usize = 1 * 1024 * 1024;
    let mut buffer = vec![0; SIZE_LIMIT];
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read < SIZE_LIMIT {
        buffer.resize(bytes_read, 0);
    }

    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join("tempfile.txt");
    let mut temp_file = File::create(&temp_file_path)?;
    temp_file.write_all(&buffer)?;

    // Create a new HTTP client
    let client = Client::new();

    // Create a multipart form with the chunk as a plain text file
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(buffer)
            .file_name("tempfile.txt")
            .mime_str("text/plain")?,
    );

    // Send the request
    let request_url = "https://node.lighthouse.storage/api/v0/add";
    println!("Sending POST request to {}", request_url);

    let response = client
        .post(request_url)
        .header("Authorization", format!("Bearer {}", lh_key))
        .multipart(form)
        .send()
        .await?;

    // Check the status
    if !response.status().is_success() {
        log::error!("OOpss! Failed to upload file. Status: {}", response.status());
        return Err("Failed to upload file".into());
    }
    let response_text = response.text().await?;
    let lighthouse_response: LighthouseResponse = serde_json::from_str(&response_text)?;
    log::info!("Your Lighthouse IPFS CID: {:?}", lighthouse_response.Hash);

    Ok(lighthouse_response)
}
