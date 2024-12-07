use crate::resource_monitor::ResourceMetrics;
use crate::container_monitor::ContainerStatus;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Clone)]
pub struct Alert {
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub source: String,
    pub message: String,
    pub details: String,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Error, Debug)]
pub enum AlertError {
    #[error("Failed to evaluate alert condition: {0}")]
    EvaluationError(String),
}

#[async_trait]
pub trait AlertRule {
    async fn evaluate(&self) -> Result<Option<Alert>, AlertError>;
}

pub struct ResourceAlertRule {
    pub threshold: ResourceThreshold,
    pub metrics: ResourceMetrics,
}

pub struct ContainerAlertRule {
    pub container: ContainerStatus,
}

#[derive(Debug)]
pub struct ResourceThreshold {
    pub cpu_threshold: f32,
    pub memory_threshold: f32,
    pub disk_threshold: f32,
}

impl ResourceAlertRule {
    pub fn new(threshold: ResourceThreshold, metrics: ResourceMetrics) -> Self {
        Self {
            threshold,
            metrics,
        }
    }
}

#[async_trait]
impl AlertRule for ResourceAlertRule {
    async fn evaluate(&self) -> Result<Option<Alert>, AlertError> {
        let cpu_usage_percent = self.metrics.cpu_usage;
        let memory_usage_percent = (self.metrics.memory_used as f32 / self.metrics.memory_total as f32) * 100.0;
        let disk_usage_percent = (self.metrics.disk_used as f32 / self.metrics.disk_total as f32) * 100.0;

        if cpu_usage_percent > self.threshold.cpu_threshold {
            return Ok(Some(Alert {
                timestamp: Utc::now(),
                severity: AlertSeverity::Warning,
                source: "CPU".to_string(),
                message: format!("High CPU usage: {:.1}%", cpu_usage_percent),
                details: format!("Threshold: {:.1}%", self.threshold.cpu_threshold),
            }));
        }

        if memory_usage_percent > self.threshold.memory_threshold {
            return Ok(Some(Alert {
                timestamp: Utc::now(),
                severity: AlertSeverity::Warning,
                source: "Memory".to_string(),
                message: format!("High memory usage: {:.1}%", memory_usage_percent),
                details: format!("Threshold: {:.1}%", self.threshold.memory_threshold),
            }));
        }

        if disk_usage_percent > self.threshold.disk_threshold {
            return Ok(Some(Alert {
                timestamp: Utc::now(),
                severity: AlertSeverity::Warning,
                source: "Disk".to_string(),
                message: format!("High disk usage: {:.1}%", disk_usage_percent),
                details: format!("Threshold: {:.1}%", self.threshold.disk_threshold),
            }));
        }

        Ok(None)
    }
}

#[async_trait]
impl AlertRule for ContainerAlertRule {
    async fn evaluate(&self) -> Result<Option<Alert>, AlertError> {
        if !self.container.running {
            return Ok(Some(Alert {
                timestamp: Utc::now(),
                severity: AlertSeverity::Critical,
                source: "Container".to_string(),
                message: format!("Container {} is not running", self.container.name),
                details: format!("Container ID: {}, Status: {}", 
                    self.container.container_id, 
                    self.container.status
                ),
            }));
        }
        Ok(None)
    }
}
