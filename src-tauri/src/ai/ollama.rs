use serde::{Deserialize, Serialize};
use crate::error::NiTriTeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatMessage {
    pub role: String,    // "system" | "user" | "assistant"
    pub content: String,
}

pub async fn chat(
    url: &str,
    model: &str,
    messages: Vec<OllamaChatMessage>,
    temperature: f64,
) -> Result<String, NiTriTeError> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": false,
        "options": { "temperature": temperature },
    });
    let resp = client
        .post(format!("{}/api/chat", url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(180))
        .send()
        .await
        .map_err(|e| NiTriTeError::OllamaUnavailable(e.to_string()))?;
    let result: serde_json::Value = resp.json().await?;
    // /api/chat returns {"message": {"role": "assistant", "content": "..."}}
    let content = result["message"]["content"].as_str().unwrap_or("").to_string();
    if content.is_empty() {
        // fallback au cas où la réponse a un format différent
        Ok(result["response"].as_str().unwrap_or("").to_string())
    } else {
        Ok(content)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OllamaModel {
    pub name: String,
    pub size_gb: f64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub url: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:11434".into(),
            model: "llama3:8b".into(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

pub async fn check_ollama(url: &str) -> bool {
    // Timeout court : reqwest::get() n'a AUCUN timeout par défaut, donc une URL
    // qui accepte la connexion sans répondre bloquerait ce health-check à vie.
    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    else {
        return false;
    };
    client.get(format!("{}/api/tags", url)).send().await.is_ok()
}

pub async fn list_models(url: &str) -> Result<Vec<OllamaModel>, NiTriTeError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let resp = client.get(format!("{}/api/tags", url)).send().await?;
    let body: serde_json::Value = resp.json().await?;

    let models = body["models"].as_array()
        .map(|arr| arr.iter().map(|m| OllamaModel {
            name: m["name"].as_str().unwrap_or("").to_string(),
            size_gb: m["size"].as_u64().unwrap_or(0) as f64 / 1_073_741_824.0,
            modified_at: m["modified_at"].as_str().unwrap_or("").to_string(),
        }).collect())
        .unwrap_or_default();

    Ok(models)
}

pub async fn query(
    url: &str,
    model: &str,
    prompt: &str,
    system_prompt: Option<&str>,
    temperature: f64,
) -> Result<String, NiTriTeError> {
    let client = reqwest::Client::new();

    let mut body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": { "temperature": temperature },
    });

    if let Some(sys) = system_prompt {
        body["system"] = serde_json::Value::String(sys.to_string());
    }

    let resp = client.post(format!("{}/api/generate", url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| NiTriTeError::OllamaUnavailable(e.to_string()))?;

    let result: serde_json::Value = resp.json().await?;
    Ok(result["response"].as_str().unwrap_or("").to_string())
}
