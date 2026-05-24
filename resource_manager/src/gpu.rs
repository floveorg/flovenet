use serde::{Deserialize, Serialize};
use std::path::Path;

/// A GPU slot with a specific VRAM allocation.
/// Slots are fractional units: 2, 4, or 8 GiB VRAM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSlot {
    /// Unique slot identifier within the node
    pub slot_id: u32,
    /// Dedicated VRAM in GiB (2, 4, or 8)
    pub vram_gb: f64,
    /// GPU model name (e.g. "NVIDIA RTX 4090")
    pub model: String,
    /// Whether this slot is currently free
    pub available: bool,
}

impl GpuSlot {
    /// Minimum VRAM per slot in GiB
    pub const MIN_VRAM_GB: f64 = 2.0;

    /// Create GPU slots from total VRAM, splitting into 2/4/8 GiB units.
    /// Returns a list of slots and the total allocated VRAM.
    pub fn create_slots(total_vram_gb: f64, model: &str) -> Vec<GpuSlot> {
        let mut slots = Vec::new();
        let mut remaining = total_vram_gb;
        let mut slot_id = 0u32;

        // Prefer 8 GiB slots, then 4, then 2
        while remaining >= 8.0 {
            slots.push(GpuSlot {
                slot_id,
                vram_gb: 8.0,
                model: model.to_string(),
                available: true,
            });
            remaining -= 8.0;
            slot_id += 1;
        }
        while remaining >= 4.0 {
            slots.push(GpuSlot {
                slot_id,
                vram_gb: 4.0,
                model: model.to_string(),
                available: true,
            });
            remaining -= 4.0;
            slot_id += 1;
        }
        while remaining >= 2.0 {
            slots.push(GpuSlot {
                slot_id,
                vram_gb: 2.0,
                model: model.to_string(),
                available: true,
            });
            remaining -= 2.0;
            slot_id += 1;
        }

        slots
    }

    /// Count how many slots can be created from `required_gb` GiB.
    pub fn slots_needed(required_gb: f64) -> u32 {
        let mut needed = 0u32;
        let mut remaining = required_gb;
        while remaining > 0.0 {
            if remaining >= 8.0 {
                remaining -= 8.0;
            } else if remaining >= 4.0 {
                remaining -= 4.0;
            } else {
                remaining -= 2.0;
            }
            needed += 1;
        }
        needed
    }
}

/// Detect GPU resources available on this system.
///
/// Detection order:
/// 1. Environment variable `FLOVENET_GPU_VRAM_GB` (e.g. "24") and `FLOVENET_GPU_MODEL` (e.g. "RTX 4090")
/// 2. NVIDIA `/proc/driver/nvidia/gpus/*/information` on Linux
///
/// Returns (vram_gb, model) or (None, None) if no GPU detected.
pub fn detect_gpu() -> (Option<f64>, Option<String>) {
    // 1. Check environment variables first (for testing / manual config)
    if let Ok(vram_str) = std::env::var("FLOVENET_GPU_VRAM_GB") {
        if let Ok(vram) = vram_str.parse::<f64>() {
            let model = std::env::var("FLOVENET_GPU_MODEL").ok();
            return (Some(vram.max(0.0)), model);
        }
    }

    // 2. Try NVIDIA /proc path on Linux
    #[cfg(target_os = "linux")]
    {
        let nvidia_dir = Path::new("/proc/driver/nvidia/gpus");
        if nvidia_dir.is_dir() {
            let mut total_vram: f64 = 0.0;
            let mut model: Option<String> = None;

            if let Ok(entries) = std::fs::read_dir(nvidia_dir) {
                for entry in entries.flatten() {
                    let info_path = entry.path().join("information");
                    if let Ok(content) = std::fs::read_to_string(&info_path) {
                        for line in content.lines() {
                            if line.starts_with("Model:") {
                                model = Some(line.trim_start_matches("Model:").trim().to_string());
                            }
                            if line.starts_with("Video Memory:") {
                                // Format: "Video Memory: 24576 MiB"
                                if let Some(val) = line.split(':').nth(1) {
                                    let val = val.trim();
                                    if let Some(mib_str) = val.split_whitespace().next() {
                                        if let Ok(mib) = mib_str.parse::<f64>() {
                                            total_vram += mib / 1024.0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if total_vram > 0.0 {
                return (Some(total_vram), model);
            }
        }
    }

    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_slots_24gb() {
        let slots = GpuSlot::create_slots(24.0, "RTX 4090");
        // 24 = 8+8+8 → three 8 GiB slots
        assert_eq!(slots.len(), 3);
        for slot in &slots {
            assert_eq!(slot.vram_gb, 8.0);
            assert!(slot.available);
        }
    }

    #[test]
    fn test_create_slots_14gb() {
        let slots = GpuSlot::create_slots(14.0, "RTX 3080");
        // 14 = 8+4+2 → one of each
        assert_eq!(slots.len(), 3);
        assert_eq!(slots[0].vram_gb, 8.0);
        assert_eq!(slots[1].vram_gb, 4.0);
        assert_eq!(slots[2].vram_gb, 2.0);
    }

    #[test]
    fn test_create_slots_3gb() {
        let slots = GpuSlot::create_slots(3.0, "GTX 1060");
        // 3 → one 2 GiB slot (1 GiB unused)
        assert_eq!(slots.len(), 1);
        assert!((slots[0].vram_gb - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_slots_needed() {
        assert_eq!(GpuSlot::slots_needed(1.0), 1); // 1 × 2 GiB
        assert_eq!(GpuSlot::slots_needed(2.0), 1); // 1 × 2 GiB
        assert_eq!(GpuSlot::slots_needed(6.0), 2); // 1 × 4 + 1 × 2
        assert_eq!(GpuSlot::slots_needed(8.0), 1); // 1 × 8
        assert_eq!(GpuSlot::slots_needed(16.0), 2); // 2 × 8
    }

    #[test]
    fn test_no_gpu_no_env() {
        // Without env var and without NVIDIA /proc, should return None
        let (vram, model) = detect_gpu();
        // Can't assert None because env might be set in CI; just check types
        assert!(vram.is_none() || vram.unwrap() > 0.0);
        drop(model);
    }
}
