use std::sync::Arc;
use std::env;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use reqwest::Client;
use crate::collector::CompleteReport;
use crate::translator;

pub struct AIManager {
    client: Client,
    api_keys: Vec<String>,
    current_model_index: Arc<Mutex<usize>>,
}

impl AIManager {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap(),
            api_keys: Self::load_api_keys(),
            current_model_index: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn analyze_with_ai(&self, report: &CompleteReport, error_context: Option<&str>, lang: &str) -> String {
        let models = vec![
            "z-ai/glm4.7",
            "deepseek-ai/deepseek-v3.2",
            "qwen/qwen3.5-397b-a17b",
            "moonshotai/kimi-k2-thinking",
        ];

        if self.api_keys.is_empty() {
            return self.local_fallback_analysis(report, lang);
        }
        
        let mut idx = *self.current_model_index.lock().await;
        let mut last_error = None;
        
        for _ in 0..models.len() {
            let model = models[idx];
            let api_key = &self.api_keys[idx % self.api_keys.len()];
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
        
        let fallback = self.local_fallback_analysis(report, lang);
        format!("{}\n{}\n{}", translator::t("error_ai_timeout"), last_error.unwrap_or_default(), fallback)
    }

    async fn call_nvidia_api(&self, model: &str, api_key: &str, report: &CompleteReport, error_context: Option<&str>, lang: &str) -> Result<String, String> {
        let prompt = self.build_analysis_prompt(report, error_context, lang);
        
        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a senior Linux SRE and security engineer. Return plain text only with this strict structure: 1) HEALTH SUMMARY, 2) CRITICAL ISSUES, 3) SECURITY RISKS, 4) PERFORMANCE RISKS, 5) TOP PRIORITY ACTIONS (numbered). Keep it concrete and command-oriented."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.4,
            "top_p": 0.9,
            "max_tokens": 1200,
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
        
        prompt.push_str("\n\nOutput requirements: prioritize by severity, no markdown table, concise actionable steps.");
        
        prompt
    }

    pub async fn get_remediation_command(&self, error_log: &str, system_context: &str) -> Option<String> {
        if self.api_keys.is_empty() {
            return None;
        }

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
            .header("Authorization", format!("Bearer {}", self.api_keys[0]))
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
        if self.api_keys.is_empty() {
            return vec![];
        }

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
        
        let response = match self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_keys[0]))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return vec![],
        };
        
        if !response.status().is_success() {
            return vec![];
        }
        
        let json_response: Value = match response.json().await {
            Ok(v) => v,
            Err(_) => return vec![],
        };
        let content = match json_response["choices"][0]["message"]["content"].as_str() {
            Some(v) => v.to_string(),
            None => return vec![],
        };
        
        content.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn load_api_keys() -> Vec<String> {
        let mut keys = Vec::new();

        for env_name in ["NVIDIA_API_KEYS", "NIM_API_KEYS"] {
            if let Ok(value) = env::var(env_name) {
                keys.extend(
                    value
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string()),
                );
            }
        }

        for env_name in ["NVIDIA_API_KEY", "NIM_API_KEY"] {
            if let Ok(value) = env::var(env_name) {
                let v = value.trim();
                if !v.is_empty() {
                    keys.push(v.to_string());
                }
            }
        }

        keys
    }

    fn local_fallback_analysis(&self, report: &CompleteReport, lang: &str) -> String {
        let mut lines = Vec::new();
        let title = if lang == "ID" {
            "ANALISIS LOKAL (AI fallback)"
        } else {
            "LOCAL ANALYSIS (AI fallback)"
        };
        lines.push(title.to_string());
        lines.push(format!(
            "{}: {}/100",
            if lang == "ID" { "Skor kesehatan" } else { "Health score" },
            report.overall_health_score
        ));

        if report.critical_issues.is_empty() {
            lines.push(if lang == "ID" {
                "Tidak ada isu kritis terdeteksi."
            } else {
                "No critical issues detected."
            }.to_string());
        } else {
            lines.push(if lang == "ID" {
                "Isu kritis:"
            } else {
                "Critical issues:"
            }.to_string());
            for issue in report.critical_issues.iter().take(5) {
                lines.push(format!("- {}", issue));
            }
        }

        lines.push(if lang == "ID" {
            "Aksi prioritas: amankan SSH, audit firewall, cek partisi kritis, optimalkan beban CPU/RAM."
        } else {
            "Priority actions: harden SSH, audit firewall, check critical partitions, optimize CPU/RAM load."
        }.to_string());

        lines.join("\n")
    }
}

impl Default for AIManager {
    fn default() -> Self {
        Self::new()
    }
}
