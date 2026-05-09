use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};

pub mod system;
pub mod hardware;
pub mod storage;
pub mod network;
pub mod security;
pub mod performance;
pub mod software;
pub mod logs;

use system::SystemCollector;
use hardware::HardwareCollector;
use storage::StorageCollector;
use network::NetworkCollector;
use security::SecurityCollector;
use performance::PerformanceCollector;
use software::SoftwareCollector;
use logs::LogsCollector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub name: String,
    pub value: String,
    pub unit: String,
    pub timestamp: DateTime<Local>,
    pub severity: MetricSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricSeverity {
    Info,
    Warning,
    Critical,
    Good,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricGroup {
    pub category: String,
    pub metrics: Vec<MetricValue>,
    pub collected_at: DateTime<Local>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteReport {
    pub system: MetricGroup,
    pub hardware: MetricGroup,
    pub storage: MetricGroup,
    pub network: MetricGroup,
    pub security: MetricGroup,
    pub performance: MetricGroup,
    pub software: MetricGroup,
    pub logs: MetricGroup,
    pub overall_health_score: u8,
    pub recommendation_count: usize,
    pub critical_issues: Vec<String>,
}

pub struct CollectorManager {
    system: SystemCollector,
    hardware: HardwareCollector,
    storage: StorageCollector,
    network: NetworkCollector,
    security: SecurityCollector,
    performance: PerformanceCollector,
    software: SoftwareCollector,
    logs: LogsCollector,
}

impl CollectorManager {
    pub fn new() -> Self {
        Self {
            system: SystemCollector::new(),
            hardware: HardwareCollector::new(),
            storage: StorageCollector::new(),
            network: NetworkCollector::new(),
            security: SecurityCollector::new(),
            performance: PerformanceCollector::new(),
            software: SoftwareCollector::new(),
            logs: LogsCollector::new(),
        }
    }

    pub async fn collect_all(&self) -> CompleteReport {
        let start = std::time::Instant::now();

        let system = self.system.collect().await;
        let hardware = self.hardware.collect().await;
        let storage = self.storage.collect().await;
        let network = self.network.collect().await;
        let security = self.security.collect().await;
        let performance = self.performance.collect().await;
        let software = self.software.collect().await;
        let logs = self.logs.collect().await;

        let critical_issues = self.extract_critical_issues(&security, &storage, &performance);
        let overall_health_score = self.calculate_health_score(&system, &hardware, &storage, &network, &security, &performance);
        let recommendation_count = security.metrics.iter()
            .filter(|m| m.severity == MetricSeverity::Warning || m.severity == MetricSeverity::Critical)
            .count() + storage.metrics.iter()
            .filter(|m| m.severity == MetricSeverity::Warning || m.severity == MetricSeverity::Critical)
            .count();

        CompleteReport {
            system,
            hardware,
            storage,
            network,
            security,
            performance,
            software,
            logs,
            overall_health_score,
            recommendation_count,
            critical_issues,
        }
    }

    fn extract_critical_issues(&self, security: &MetricGroup, storage: &MetricGroup, performance: &MetricGroup) -> Vec<String> {
        let mut issues = Vec::new();

        for metric in &security.metrics {
            if metric.severity == MetricSeverity::Critical {
                issues.push(format!("Security: {}", metric.value));
            }
        }

        for metric in &storage.metrics {
            if metric.severity == MetricSeverity::Critical && metric.name.contains("usage") {
                issues.push(format!("Storage: {}", metric.value));
            }
        }

        for metric in &performance.metrics {
            if metric.severity == MetricSeverity::Critical {
                issues.push(format!("Performance: {}", metric.value));
            }
        }

        issues
    }

    fn calculate_health_score(&self, system: &MetricGroup, hardware: &MetricGroup, storage: &MetricGroup, network: &MetricGroup, security: &MetricGroup, performance: &MetricGroup) -> u8 {
        let mut score = 100;

        let security_warnings = security.metrics.iter()
            .filter(|m| m.severity == MetricSeverity::Warning).count();
        let security_criticals = security.metrics.iter()
            .filter(|m| m.severity == MetricSeverity::Critical).count();
        score = score.saturating_sub((security_warnings * 5) as u8);
        score = score.saturating_sub((security_criticals * 15) as u8);

        let storage_warnings = storage.metrics.iter()
            .filter(|m| m.severity == MetricSeverity::Warning).count();
        let storage_criticals = storage.metrics.iter()
            .filter(|m| m.severity == MetricSeverity::Critical).count();
        score = score.saturating_sub((storage_warnings * 3) as u8);
        score = score.saturating_sub((storage_criticals * 10) as u8);

        let perf_warnings = performance.metrics.iter()
            .filter(|m| m.severity == MetricSeverity::Warning).count();
        let perf_criticals = performance.metrics.iter()
            .filter(|m| m.severity == MetricSeverity::Critical).count();
        score = score.saturating_sub((perf_warnings * 4) as u8);
        score = score.saturating_sub((perf_criticals * 12) as u8);

        let cpu_usage: f32 = hardware.metrics.iter()
            .find(|m| m.name == "cpu_usage_percent")
            .and_then(|m| m.value.parse().ok())
            .unwrap_or(0.0);
        if cpu_usage > 90.0 {
            score = score.saturating_sub(10);
        } else if cpu_usage > 75.0 {
            score = score.saturating_sub(5);
        }

        let ram_usage: f32 = hardware.metrics.iter()
            .find(|m| m.name == "ram_usage_percent")
            .and_then(|m| m.value.parse().ok())
            .unwrap_or(0.0);
        if ram_usage > 90.0 {
            score = score.saturating_sub(10);
        } else if ram_usage > 75.0 {
            score = score.saturating_sub(5);
        }

        score.max(0).min(100)
    }
}

impl Default for CollectorManager {
    fn default() -> Self {
        Self::new()
    }
}