use async_trait::async_trait;
use sysinfo::{System, SystemExt, CpuExt};
use thiserror::Error;

#[derive(Debug)]
pub struct ResourceMetrics {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
}

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("Failed to collect system metrics: {0}")]
    MetricsCollectionError(String),
}

#[async_trait]
pub trait ResourceMonitor {
    async fn collect_metrics(&self) -> Result<ResourceMetrics, ResourceError>;
}

pub struct SystemResourceMonitor {
    sys: System,
}

impl SystemResourceMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }
}

#[async_trait]
impl ResourceMonitor for SystemResourceMonitor {
    async fn collect_metrics(&self) -> Result<ResourceMetrics, ResourceError> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
        
        Ok(ResourceMetrics {
            cpu_usage,
            memory_used: sys.used_memory(),
            memory_total: sys.total_memory(),
            disk_used: sys.disks().iter().map(|disk| disk.total_space() - disk.available_space()).sum(),
            disk_total: sys.disks().iter().map(|disk| disk.total_space()).sum(),
        })
    }
}
