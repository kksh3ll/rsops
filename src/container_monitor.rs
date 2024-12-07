use async_trait::async_trait;
use bollard::Docker;
use bollard::container::ListContainersOptions;
use thiserror::Error;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContainerStatus {
    pub container_id: String,
    pub name: String,
    pub status: String,
    pub running: bool,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f64>,
}

#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("Docker connection error: {0}")]
    ConnectionError(String),
    #[error("Container monitoring error: {0}")]
    MonitoringError(String),
}

#[async_trait]
pub trait ContainerMonitor {
    async fn list_containers(&self) -> Result<Vec<ContainerStatus>, ContainerError>;
    async fn get_container_stats(&self, container_id: &str) -> Result<ContainerStatus, ContainerError>;
}

pub struct DockerContainerMonitor {
    docker: Docker,
}

impl DockerContainerMonitor {
    pub fn new() -> Result<Self, ContainerError> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| ContainerError::ConnectionError(e.to_string()))?;
        
        Ok(Self { docker })
    }
}

#[async_trait]
impl ContainerMonitor for DockerContainerMonitor {
    async fn list_containers(&self) -> Result<Vec<ContainerStatus>, ContainerError> {
        let options = Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        });

        let containers = self.docker
            .list_containers(options)
            .await
            .map_err(|e| ContainerError::MonitoringError(e.to_string()))?;

        let mut container_statuses = Vec::new();
        for container in containers {
            let id = container.id.unwrap_or_default();
            let name = container.names.unwrap_or_default()
                .first()
                .cloned()
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string();
            let status = container.status.unwrap_or_default();
            let running = status.to_lowercase().contains("up");

            container_statuses.push(ContainerStatus {
                container_id: id,
                name,
                status,
                running,
                memory_usage: None,
                cpu_usage: None,
            });
        }

        Ok(container_statuses)
    }

    async fn get_container_stats(&self, container_id: &str) -> Result<ContainerStatus, ContainerError> {
        let stats = self.docker
            .stats_once(container_id)
            .await
            .map_err(|e| ContainerError::MonitoringError(e.to_string()))?;

        let container = self.docker
            .inspect_container(container_id, None)
            .await
            .map_err(|e| ContainerError::MonitoringError(e.to_string()))?;

        let name = container.name.unwrap_or_default()
            .trim_start_matches('/')
            .to_string();
        
        let status = container.state
            .and_then(|s| s.status)
            .unwrap_or_default();
        
        let running = container.state
            .and_then(|s| s.running)
            .unwrap_or(false);

        Ok(ContainerStatus {
            container_id: container_id.to_string(),
            name,
            status,
            running,
            memory_usage: Some(stats.memory_stats.usage.unwrap_or(0)),
            cpu_usage: Some(stats.cpu_stats.cpu_usage.total_usage as f64),
        })
    }
}
