use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use reqwest::Client;
use crate::collector::CompleteReport;
use crate::translator;

pub struct AIManager {
    client: Client,
    api_key_nvidia1: String,
    api_key_nvidia2: String,
    api_key_nvidia3: String,
    api_key_nvidia4: String,
    current_model_index: Arc<Mutex<usize>>,
}

impl AIManager {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap(),
            api_key_nvidia1: "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".to_string(),
            api_key_nvidia2: "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".to_string(),
            api_key_nvidia3: "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".to_string(),
            api_key_nvidia4: "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".to_string(),
            current_model_index: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn analyze_with_ai(&self, report: &CompleteReport, error_context: Option<&str>, lang: &str) -> String {
        let models = vec![
            ("z-ai/glm4.7", &self.api_key_nvidia1),
            ("deepseek-ai/deepseek-v3.2", &self.api_key_nvidia2),
            ("qwen/qwen3.5-397b-a17b", &self.api_key_nvidia3),
            ("moonshotai/kimi-k2-thinking", &self.api_key_nvidia4),
        ];
        
        let mut idx = *self.current_model_index.lock().await;
        let mut last_error = None;
        
        for _ in 0..models.len() {
            let (model, api_key) = models[idx];
            match self.call_nvidia_api(model, api_key, report, error_context, lang).await {
                Ok(result) => {
                    let mut lock = self.current_model_index.lock().await;
                    *lock = (idx + 1) % models.len();
                    return result;
                }
                Err(e) => {
                    last_error = Some(e);
                    idx = (idx + 1) % models.len();
                }
            }
        }
        
        format!("{} {}", translator::t("error_ai_timeout"), last_error.unwrap_or_default())
    }

    async fn call_nvidia_api(&self, model: &str, api_key: &str, report: &CompleteReport, error_context: Option<&str>, lang: &str) -> Result<String, String> {
        let prompt = self.build_analysis_prompt(report, error_context, lang);
        
        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a VPS system expert and DevOps engineer. Analyze the VPS data and provide: 1) Critical issues summary, 2) Performance bottlenecks, 3) Security vulnerabilities, 4) Specific actionable recommendations. Be concise but thorough. Return ONLY the analysis text."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
            "top_p": 0.95,
            "max_tokens": 4096,
            "stream": false
        });
        
        let url = "https://integrate.api.nvidia.com/v1/chat/completions";
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;
        
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

    fn build_analysis_prompt(&self, report: &CompleteReport, error_context: Option<&str>, lang: &str) -> String {
        let mut prompt = String::new();
        
        if let Some(ctx) = error_context {
            prompt.push_str(&format!("ERROR CONTEXT:\n{}\n\n", ctx));
        }
        
        prompt.push_str(&format!("SYSTEM HEALTH SCORE: {}/100\n\n", report.overall_health_score));
        
        prompt.push_str("CRITICAL ISSUES:\n");
        for issue in &report.critical_issues {
            prompt.push_str(&format!("- {}\n", issue));
        }
        
        prompt.push_str("\nSECURITY METRICS:\n");
        for metric in &report.security.metrics {
            if metric.severity == crate::collector::MetricSeverity::Critical || metric.severity == crate::collector::MetricSeverity::Warning {
                prompt.push_str(&format!("- {}: {} [{}]\n", metric.name, metric.value, 
                    if metric.severity == crate::collector::MetricSeverity::Critical { "CRITICAL" } else { "WARNING" }));
            }
        }
        
        prompt.push_str("\nPERFORMANCE METRICS:\n");
        for metric in &report.performance.metrics {
            if metric.severity == crate::collector::MetricSeverity::Critical || metric.severity == crate::collector::MetricSeverity::Warning {
                prompt.push_str(&format!("- {}: {} {}\n", metric.name, metric.value, metric.unit));
            }
        }
        
        prompt.push_str("\nSTORAGE STATUS:\n");
        for metric in &report.storage.metrics {
            if metric.severity == crate::collector::MetricSeverity::Critical || metric.severity == crate::collector::MetricSeverity::Warning {
                prompt.push_str(&format!("- {}: {}\n", metric.name, metric.value));
            }
        }
        
        let cpu_metric = report.hardware.metrics.iter().find(|m| m.name == "cpu_usage_percent");
        let ram_metric = report.hardware.metrics.iter().find(|m| m.name == "ram_usage_percent");
        
        if let Some(cpu) = cpu_metric {
            prompt.push_str(&format!("\nCPU Usage: {}%\n", cpu.value));
        }
        if let Some(ram) = ram_metric {
            prompt.push_str(&format!("RAM Usage: {}%\n", ram.value));
        }
        
        prompt.push_str(&format!("\nTotal Recommendations Needed: {}\n", report.recommendation_count));
        
        if lang == "ID" {
            prompt.push_str("\nBerikan analisis dalam Bahasa Indonesia.");
        }
        
        prompt.push_str("\n\nProvide structured analysis with numbered recommendations. Prioritize by severity.");
        
        prompt
    }

    pub async fn get_remediation_command(&self, error_log: &str, system_context: &str) -> Option<String> {
        let prompt = format!(
            "SYSTEM CONTEXT: {}\n\nERROR LOG (raw):\n{}\n\nBased on this error log, provide EXACTLY ONE bash command that would fix this issue. \nRules:\n1. Return ONLY the command, no explanation\n2. Must be safe to run\n3. Use standard Linux utilities\n4. If cannot determine, return: echo 'FIX_NOT_FOUND'\n\nCommand:",
            system_context,
            error_log
        );
        
        let payload = json!({
            "model": "deepseek-ai/deepseek-v3.2",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a Linux system administrator. Output only bash commands without markdown or explanation."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 256,
            "stream": false
        });
        
        let url = "https://integrate.api.nvidia.com/v1/chat/completions";
        
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key_nvidia2))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .ok()?;
        
        if !response.status().is_success() {
            return None;
        }
        
        let json_response: Value = response.json().await.ok()?;
        let command = json_response["choices"][0]["message"]["content"]
            .as_str()?
            .trim()
            .to_string();
        
        if command.contains("FIX_NOT_FOUND") || command.len() > 500 {
            None
        } else {
            Some(command)
        }
    }

    pub async fn suggest_alternative_commands(&self, failed_command: &str, error_output: &str) -> Vec<String> {
        let prompt = format!(
            "Failed command: {}\nError output: {}\n\nSuggest 3 alternative ways to achieve the same goal. Return as comma-separated list of commands.",
            failed_command,
            error_output
        );
        
        let payload = json!({
            "model": "qwen/qwen3.5-397b-a17b",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.6,
            "max_tokens": 512,
        });
        
        let url = "https://integrate.api.nvidia.com/v1/chat/completions";
        
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key_nvidia3))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .ok()?;
        
        if !response.status().is_success() {
            return vec![];
        }
        
        let json_response: Value = response.json().await.ok()?;
        let content = json_response["choices"][0]["message"]["content"]
            .as_str()?
            .to_string();
        
        content.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl Default for AIManager {
    fn default() -> Self {
        Self::new()
    }
}