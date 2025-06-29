use reqwest::blocking::Client;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Testing TLS connection...");

    let client = Client::builder()
        
        .build().expect("Build failed");

    let response = client.get("https://www.google.com").send().expect("Get failed");

    println!("Response status: {}", response.status());
    Ok(())
}
