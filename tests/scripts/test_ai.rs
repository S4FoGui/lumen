use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}
#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}
#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        // Tenta ler do config.yaml se não estiver no env
        "AIzaSyCpqJ6fBS55kBC_3s7N6QsLITeRaAn7H78".to_string() // Substituirei pelo valor real do seu config
    });
    
    let client = reqwest::Client::new();
    let prompt = "Responda apenas com a palavra 'FUNCIONOU'.";
    
    let request = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: prompt.to_string() }],
        }],
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    let response = client.post(&url).json(&request).send().await?;
    let status = response.status();
    let body = response.text().await?;
    
    println!("Status: {}", status);
    println!("Corpo da Resposta: {}", body);
    
    Ok(())
}
