use std::time::Duration;
use serde_json::{json, Value};
use reqwest::Client;
use tokio::time::timeout;

pub struct NvidiaClient {
    client: Client,
    api_keys: Vec<String>,
    base_url: String,
}

impl NvidiaClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(90))
                .build()
                .unwrap(),
            api_keys: vec![
                "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".to_string(),
                "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".to_string(),
            ],
            base_url: "https://integrate.api.nvidia.com/v1".to_string(),
        }
    }

    pub async fn chat_completion(&self, model: &str, messages: Vec<(String, String)>, temperature: f64, max_tokens: u32) -> Result<String, String> {
        let api_key = &self.api_keys[0];
        
        let formatted_messages: Vec<Value> = messages.iter()
            .map(|(role, content)| json!({
                "role": role,
                "content": content
            }))
            .collect();
        
        let payload = json!({
            "model": model,
            "messages": formatted_messages,
            "temperature": temperature,
            "top_p": 0.95,
            "max_tokens": max_tokens,
            "stream": false
        });
        
        let url = format!("{}/chat/completions", self.base_url);
        
        let response = timeout(Duration::from_secs(60), async {
            self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
        }).await.map_err(|_| "Request timeout".to_string())?
          .map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, error_text));
        }
        
        let json_response: Value = response.json().await.map_err(|e| format!("Parse error: {}", e))?;
        
        let content = json_response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in response")?
            .to_string();
        
        Ok(content)
    }

    pub async fn chat_completion_streaming(&self, model: &str, messages: Vec<(String, String)>, temperature: f64, on_chunk: impl Fn(&str) + Send) -> Result<(), String> {
        let api_key = &self.api_keys[0];
        
        let formatted_messages: Vec<Value> = messages.iter()
            .map(|(role, content)| json!({
                "role": role,
                "content": content
            }))
            .collect();
        
        let payload = json!({
            "model": model,
            "messages": formatted_messages,
            "temperature": temperature,
            "top_p": 0.95,
            "max_tokens": 4096,
            "stream": true
        });
        
        let url = format!("{}/chat/completions", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, error_text));
        }
        
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        let mut buffer = String::new();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);
            
            for line in buffer.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        return Ok(());
                    }
                    if let Ok(json_value) = serde_json::from_str::<Value>(data) {
                        if let Some(content) = json_value["choices"][0]["delta"]["content"].as_str() {
                            on_chunk(content);
                        }
                        if let Some(reasoning) = json_value["choices"][0]["delta"]["reasoning_content"].as_str() {
                            on_chunk(reasoning);
                        }
                    }
                }
            }
            buffer.clear();
        }
        
        Ok(())
    }

    pub async fn generate_analysis(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let messages = vec![
            ("system".to_string(), system_prompt.to_string()),
            ("user".to_string(), user_prompt.to_string()),
        ];
        
        self.chat_completion("deepseek-ai/deepseek-v3.2", messages, 0.7, 4096).await
    }

    pub async fn generate_command(&self, error_context: &str, system_info: &str) -> Result<String, String> {
        let system_prompt = "You are a Linux system administrator expert. Based on the error and system info, provide EXACTLY ONE bash command that will fix the issue. Return ONLY the command, no explanation, no markdown. If unsure, return: echo 'UNSURE'";
        
        let user_prompt = format!("System: {}\n\nError: {}\n\nCommand:", system_info, error_context);
        
        let messages = vec![
            ("system".to_string(), system_prompt.to_string()),
            ("user".to_string(), user_prompt),
        ];
        
        let result = self.chat_completion("z-ai/glm4.7", messages, 0.3, 256).await?;
        
        if result.contains("UNSURE") || result.len() > 300 {
            Ok("echo 'No automated fix available'".to_string())
        } else {
            Ok(result.trim().to_string())
        }
    }

    pub async fn generate_report_summary(&self, report_data: &str, language: &str) -> Result<String, String> {
        let system_prompt = if language == "ID" {
            "Anda adalah ahli DevOps. Buat ringkasan eksekutif dari laporan VPS dalam Bahasa Indonesia. Sertakan: skor kesehatan, masalah kritis, rekomendasi prioritas. Maksimal 500 kata."
        } else {
            "You are a DevOps expert. Create an executive summary of the VPS report. Include: health score, critical issues, top recommendations. Max 500 words."
        };
        
        let user_prompt = format!("Report data:\n{}", report_data);
        
        let messages = vec![
            ("system".to_string(), system_prompt.to_string()),
            ("user".to_string(), user_prompt),
        ];
        
        self.chat_completion("qwen/qwen3.5-397b-a17b", messages, 0.6, 2048).await
    }

    pub async fn suggest_altenatives(&self, failed_command: &str, error_output: &str) -> Result<Vec<String>, String> {
        let prompt = format!("Failed command: {}\nError: {}\n\nSuggest 3 alternative ways to achieve the same goal.", failed_command, error_output);
        
        let messages = vec![
            ("system".to_string(), "List 3 alternatives as comma-separated commands. No explanations.".to_string()),
            ("user".to_string(), prompt),
        ];
        
        let result = self.chat_completion("moonshotai/kimi-k2-thinking", messages, 0.5, 512).await?;
        
        let alternatives: Vec<String> = result.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.len() < 200)
            .collect();
        
        if alternatives.is_empty() {
            Ok(vec!["echo 'No alternatives found'".to_string()])
        } else {
            Ok(alternatives)
        }
    }

    pub async fn detect_anomalies(&self, metrics: &str) -> Result<String, String> {
        let system_prompt = "Analyze the system metrics and detect anomalies. Return JSON format: {\"anomalies\": [{\"metric\": \"name\", \"value\": \"current\", \"expected\": \"normal\", \"severity\": \"high/medium/low\"}]}";
        
        let user_prompt = format!("Metrics:\n{}", metrics);
        
        let messages = vec![
            ("system".to_string(), system_prompt.to_string()),
            ("user".to_string(), user_prompt),
        ];
        
        self.chat_completion("deepseek-ai/deepseek-v3.2", messages, 0.4, 2048).await
    }
}

impl Default for NvidiaClient {
    fn default() -> Self {
        Self::new()
    }
}