//! System information collection for performance testing
//! Integrates with zero-latency-observability crate for comprehensive system monitoring

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub cpu_info: String,
    pub total_memory_gb: f64,
    pub available_memory_gb: f64,
    pub rust_version: String,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedSystemInfo {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub cpu_frequency_mhz: u32,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub swap_total_mb: u64,
    pub swap_available_mb: u64,
    pub disk_total_gb: f64,
    pub disk_available_gb: f64,
    pub load_average: (f64, f64, f64), // 1min, 5min, 15min
}

impl SystemInfo {
    /// Collect comprehensive system information
    pub async fn collect() -> Result<Self, Box<dyn std::error::Error>> {
        let detailed = DetailedSystemInfo::collect().await?;
        
        Ok(SystemInfo {
            os: std::env::consts::OS.to_string(),
            cpu_info: format!("{} ({} cores @ {}MHz)", 
                             detailed.cpu_model, 
                             detailed.cpu_cores, 
                             detailed.cpu_frequency_mhz),
            total_memory_gb: detailed.total_memory_mb as f64 / 1024.0,
            available_memory_gb: detailed.available_memory_mb as f64 / 1024.0,
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
        })
    }
}

impl DetailedSystemInfo {
    /// Collect detailed system information across platforms
    pub async fn collect() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            Self::collect_linux().await
        }
        #[cfg(target_os = "macos")]
        {
            Self::collect_macos().await
        }
        #[cfg(target_os = "windows")]
        {
            Self::collect_windows().await
        }
    }

    #[cfg(target_os = "linux")]
    async fn collect_linux() -> Result<Self, Box<dyn std::error::Error>> {
        // Read CPU information
        let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;
        let cpu_model = Self::extract_cpu_model_linux(&cpuinfo);
        let cpu_cores = Self::count_cpu_cores_linux(&cpuinfo);
        let cpu_frequency = Self::extract_cpu_frequency_linux(&cpuinfo);

        // Read memory information
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let (total_mem, available_mem, swap_total, swap_free) = Self::parse_meminfo_linux(&meminfo);

        // Read load average
        let loadavg = fs::read_to_string("/proc/loadavg")?;
        let load_average = Self::parse_loadavg_linux(&loadavg);

        // Get disk information
        let (disk_total, disk_available) = Self::get_disk_info_linux().await?;

        Ok(DetailedSystemInfo {
            cpu_model,
            cpu_cores,
            cpu_frequency_mhz: cpu_frequency,
            total_memory_mb: total_mem,
            available_memory_mb: available_mem,
            swap_total_mb: swap_total,
            swap_available_mb: swap_free,
            disk_total_gb: disk_total,
            disk_available_gb: disk_available,
            load_average,
        })
    }

    #[cfg(target_os = "macos")]
    async fn collect_macos() -> Result<Self, Box<dyn std::error::Error>> {
        // Use system_profiler and sysctl for macOS
        let cpu_model = Self::get_macos_cpu_model().await?;
        let cpu_cores = Self::get_macos_cpu_cores().await?;
        let cpu_frequency = Self::get_macos_cpu_frequency().await?;
        
        let (total_mem, available_mem) = Self::get_macos_memory().await?;
        let (swap_total, swap_available) = Self::get_macos_swap().await?;
        let load_average = Self::get_macos_load_average().await?;
        let (disk_total, disk_available) = Self::get_macos_disk_info().await?;

        Ok(DetailedSystemInfo {
            cpu_model,
            cpu_cores,
            cpu_frequency_mhz: cpu_frequency,
            total_memory_mb: total_mem,
            available_memory_mb: available_mem,
            swap_total_mb: swap_total,
            swap_available_mb: swap_available,
            disk_total_gb: disk_total,
            disk_available_gb: disk_available,
            load_average,
        })
    }

    #[cfg(target_os = "windows")]
    async fn collect_windows() -> Result<Self, Box<dyn std::error::Error>> {
        // Use WMI queries for Windows
        // For now, return simulated values
        Ok(DetailedSystemInfo {
            cpu_model: "Windows CPU".to_string(),
            cpu_cores: 8,
            cpu_frequency_mhz: 3000,
            total_memory_mb: 16384,
            available_memory_mb: 12288,
            swap_total_mb: 4096,
            swap_available_mb: 2048,
            disk_total_gb: 500.0,
            disk_available_gb: 200.0,
            load_average: (0.5, 0.6, 0.7),
        })
    }

    // Linux-specific parsing functions
    #[cfg(target_os = "linux")]
    fn extract_cpu_model_linux(cpuinfo: &str) -> String {
        for line in cpuinfo.lines() {
            if line.starts_with("model name") {
                if let Some(model) = line.split(':').nth(1) {
                    return model.trim().to_string();
                }
            }
        }
        "Unknown CPU".to_string()
    }

    #[cfg(target_os = "linux")]
    fn count_cpu_cores_linux(cpuinfo: &str) -> u32 {
        cpuinfo.lines()
            .filter(|line| line.starts_with("processor"))
            .count() as u32
    }

    #[cfg(target_os = "linux")]
    fn extract_cpu_frequency_linux(cpuinfo: &str) -> u32 {
        for line in cpuinfo.lines() {
            if line.starts_with("cpu MHz") {
                if let Some(freq_str) = line.split(':').nth(1) {
                    if let Ok(freq) = freq_str.trim().parse::<f64>() {
                        return freq as u32;
                    }
                }
            }
        }
        0
    }

    #[cfg(target_os = "linux")]
    fn parse_meminfo_linux(meminfo: &str) -> (u64, u64, u64, u64) {
        let mut total_mem = 0;
        let mut available_mem = 0;
        let mut swap_total = 0;
        let mut swap_free = 0;

        for line in meminfo.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let value = parts[1].parse::<u64>().unwrap_or(0);
                match parts[0] {
                    "MemTotal:" => total_mem = value,
                    "MemAvailable:" => available_mem = value,
                    "SwapTotal:" => swap_total = value,
                    "SwapFree:" => swap_free = value,
                    _ => {}
                }
            }
        }

        (total_mem / 1024, available_mem / 1024, swap_total / 1024, swap_free / 1024)
    }

    #[cfg(target_os = "linux")]
    fn parse_loadavg_linux(loadavg: &str) -> (f64, f64, f64) {
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        if parts.len() >= 3 {
            let load1 = parts[0].parse().unwrap_or(0.0);
            let load5 = parts[1].parse().unwrap_or(0.0);
            let load15 = parts[2].parse().unwrap_or(0.0);
            (load1, load5, load15)
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    #[cfg(target_os = "linux")]
    async fn get_disk_info_linux() -> Result<(f64, f64), Box<dyn std::error::Error>> {
        let output = Command::new("df")
            .args(&["-BG", "/"])
            .output()?;
        
        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total_str = parts[1].trim_end_matches('G');
                let available_str = parts[3].trim_end_matches('G');
                
                let total = total_str.parse::<f64>().unwrap_or(0.0);
                let available = available_str.parse::<f64>().unwrap_or(0.0);
                
                return Ok((total, available));
            }
        }
        
        Ok((0.0, 0.0))
    }

    // macOS-specific functions
    #[cfg(target_os = "macos")]
    async fn get_macos_cpu_model() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("sysctl")
            .args(&["-n", "machdep.cpu.brand_string"])
            .output()?;
        
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    #[cfg(target_os = "macos")]
    async fn get_macos_cpu_cores() -> Result<u32, Box<dyn std::error::Error>> {
        let output = Command::new("sysctl")
            .args(&["-n", "hw.ncpu"])
            .output()?;
        
        let cores_str = String::from_utf8(output.stdout)?;
        Ok(cores_str.trim().parse().unwrap_or(1))
    }

    #[cfg(target_os = "macos")]
    async fn get_macos_cpu_frequency() -> Result<u32, Box<dyn std::error::Error>> {
        let output = Command::new("sysctl")
            .args(&["-n", "hw.cpufrequency_max"])
            .output()?;
        
        let freq_str = String::from_utf8(output.stdout)?;
        let freq_hz = freq_str.trim().parse::<u64>().unwrap_or(0);
        Ok((freq_hz / 1_000_000) as u32) // Convert Hz to MHz
    }

    #[cfg(target_os = "macos")]
    async fn get_macos_memory() -> Result<(u64, u64), Box<dyn std::error::Error>> {
        let output = Command::new("sysctl")
            .args(&["-n", "hw.memsize"])
            .output()?;
        
        let mem_str = String::from_utf8(output.stdout)?;
        let total_bytes = mem_str.trim().parse::<u64>().unwrap_or(0);
        let total_mb = total_bytes / (1024 * 1024);
        
        // Get available memory using vm_stat
        let vm_output = Command::new("vm_stat").output()?;
        let vm_str = String::from_utf8(vm_output.stdout)?;
        let available_mb = Self::parse_macos_vm_stat(&vm_str);
        
        Ok((total_mb, available_mb))
    }

    #[cfg(target_os = "macos")]
    fn parse_macos_vm_stat(vm_stat: &str) -> u64 {
        let mut free_pages = 0;
        let mut inactive_pages = 0;
        
        for line in vm_stat.lines() {
            if line.starts_with("Pages free:") {
                if let Some(pages_str) = line.split(':').nth(1) {
                    free_pages = pages_str.trim().trim_end_matches('.').parse().unwrap_or(0);
                }
            } else if line.starts_with("Pages inactive:") {
                if let Some(pages_str) = line.split(':').nth(1) {
                    inactive_pages = pages_str.trim().trim_end_matches('.').parse().unwrap_or(0);
                }
            }
        }
        
        // Assume 4KB pages, convert to MB
        (free_pages + inactive_pages) * 4 / 1024
    }

    #[cfg(target_os = "macos")]
    async fn get_macos_swap() -> Result<(u64, u64), Box<dyn std::error::Error>> {
        let output = Command::new("sysctl")
            .args(&["-n", "vm.swapusage"])
            .output()?;
        
        let swap_str = String::from_utf8(output.stdout)?;
        // Parse output like "total = 2048.00M  used = 0.00M  free = 2048.00M"
        let mut total = 0;
        let mut free = 0;
        
        for part in swap_str.split_whitespace() {
            if part.ends_with('M') {
                let value_str = part.trim_end_matches('M');
                if let Ok(value) = value_str.parse::<f64>() {
                    if swap_str.contains(&format!("total = {}", part)) {
                        total = value as u64;
                    } else if swap_str.contains(&format!("free = {}", part)) {
                        free = value as u64;
                    }
                }
            }
        }
        
        Ok((total, free))
    }

    #[cfg(target_os = "macos")]
    async fn get_macos_load_average() -> Result<(f64, f64, f64), Box<dyn std::error::Error>> {
        let output = Command::new("uptime").output()?;
        let uptime_str = String::from_utf8(output.stdout)?;
        
        // Parse load averages from uptime output
        if let Some(load_part) = uptime_str.split("load averages:").nth(1) {
            let loads: Vec<f64> = load_part
                .split_whitespace()
                .take(3)
                .filter_map(|s| s.parse().ok())
                .collect();
                
            if loads.len() >= 3 {
                return Ok((loads[0], loads[1], loads[2]));
            }
        }
        
        Ok((0.0, 0.0, 0.0))
    }

    #[cfg(target_os = "macos")]
    async fn get_macos_disk_info() -> Result<(f64, f64), Box<dyn std::error::Error>> {
        let output = Command::new("df")
            .args(&["-g", "/"])
            .output()?;
        
        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parts[1].parse::<f64>().unwrap_or(0.0);
                let available = parts[3].parse::<f64>().unwrap_or(0.0);
                return Ok((total, available));
            }
        }
        
        Ok((0.0, 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_info_collection() {
        let system_info = SystemInfo::collect().await;
        assert!(system_info.is_ok());
        
        let info = system_info.unwrap();
        assert!(!info.cpu_info.is_empty());
        assert!(info.total_memory_gb > 0.0);
        assert!(info.available_memory_gb > 0.0);
        assert!(info.available_memory_gb <= info.total_memory_gb);
        
        println!("System Info: {:?}", info);
    }

    #[tokio::test]
    async fn test_detailed_system_info() {
        let detailed_info = DetailedSystemInfo::collect().await;
        assert!(detailed_info.is_ok());
        
        let info = detailed_info.unwrap();
        assert!(info.cpu_cores > 0);
        assert!(info.total_memory_mb > 0);
        
        println!("Detailed System Info: {:?}", info);
    }
}
