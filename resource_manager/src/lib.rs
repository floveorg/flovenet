pub mod gpu;

use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    pub cpu_cores: u32,
    pub cpu_freq_mhz: f64,
    pub ram_total_gb: f64,
    pub ram_available_gb: f64,
    pub disk_total_gb: f64,
    pub disk_available_gb: f64,
    pub gpu_vram_gb: Option<f64>,
    pub gpu_model: Option<String>,
    pub uptime_secs: u64,
}

impl NodeResources {
    pub fn detect() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        sys.refresh_all();

        let cpu_cores = sys.cpus().len() as u32;
        let cpu_freq = sys.cpus().first().map(|c| c.frequency() as f64).unwrap_or(0.0);
        let ram_total_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let ram_avail_gb = sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

        let disks = Disks::new_with_refreshed_list();
        let (disk_total, disk_avail) = {
            let mut total = 0u64;
            let mut avail = 0u64;
            for d in &disks {
                total += d.total_space();
                avail += d.available_space();
            }
            (total as f64 / 1024.0 / 1024.0 / 1024.0, avail as f64 / 1024.0 / 1024.0 / 1024.0)
        };

        let uptime = System::uptime();
        let (gpu_vram_gb, gpu_model) = gpu::detect_gpu();

        Self {
            cpu_cores,
            cpu_freq_mhz: cpu_freq,
            ram_total_gb: (ram_total_gb * 10.0).round() / 10.0,
            ram_available_gb: (ram_avail_gb * 10.0).round() / 10.0,
            disk_total_gb: (disk_total * 10.0).round() / 10.0,
            disk_available_gb: (disk_avail * 10.0).round() / 10.0,
            gpu_vram_gb,
            gpu_model,
            uptime_secs: uptime,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeRole {
    Storage,
    Validation,
    Compute,
    Ai,
    Social,
}

impl NodeRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeRole::Storage => "storage",
            NodeRole::Validation => "validation",
            NodeRole::Compute => "compute",
            NodeRole::Ai => "ai",
            NodeRole::Social => "social",
        }
    }
}

impl std::str::FromStr for NodeRole {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "storage" => Ok(NodeRole::Storage),
            "validation" => Ok(NodeRole::Validation),
            "compute" => Ok(NodeRole::Compute),
            "ai" => Ok(NodeRole::Ai),
            "social" => Ok(NodeRole::Social),
            _ => Err(format!("unknown role: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDescriptor {
    pub peer_id: String,
    pub roles: Vec<NodeRole>,
    pub resources: NodeResources,
    pub region: String,
    pub api_url: Option<String>,
    pub total_slots: u32,
    pub available_slots: u32,
}

impl NodeDescriptor {
    pub fn slots_for_role(role: &NodeRole, resources: &NodeResources) -> u32 {
        let cpu_slots = (resources.cpu_cores).max(1);
        match role {
            NodeRole::Storage => cpu_slots.min(16),
            NodeRole::Validation => cpu_slots.min(8),
            NodeRole::Compute => cpu_slots.min(32),
            NodeRole::Ai => {
                // Ai slots limited by both CPU and GPU VRAM (2 GiB per slot minimum)
                let vram_slots = resources
                    .gpu_vram_gb
                    .map(|v| (v / gpu::GpuSlot::MIN_VRAM_GB) as u32)
                    .unwrap_or(0)
                    .max(1); // at least 1 slot for CPU-only AI tasks
                cpu_slots.min(8).min(vram_slots)
            }
            NodeRole::Social => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_role_as_str() {
        assert_eq!(NodeRole::Storage.as_str(), "storage");
        assert_eq!(NodeRole::Ai.as_str(), "ai");
        assert_eq!(NodeRole::Social.as_str(), "social");
    }

    #[test]
    fn test_node_role_from_str() {
        assert_eq!("storage".parse::<NodeRole>().unwrap(), NodeRole::Storage);
        assert!("unknown".parse::<NodeRole>().is_err());
    }

    #[test]
    fn test_slots_for_role() {
        let res = NodeResources {
            cpu_cores: 8,
            cpu_freq_mhz: 2400.0,
            ram_total_gb: 32.0,
            ram_available_gb: 16.0,
            disk_total_gb: 500.0,
            disk_available_gb: 200.0,
            gpu_vram_gb: None,
            gpu_model: None,
            uptime_secs: 3600,
        };
        assert_eq!(NodeDescriptor::slots_for_role(&NodeRole::Compute, &res), 8);
        assert_eq!(NodeDescriptor::slots_for_role(&NodeRole::Social, &res), 1);
    }
}
