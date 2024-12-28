use std::thread;
use std::time::Duration;
use sysinfo::{CpuExt, System, SystemExt};

use instance_monitor::metric_registry::{MetricRegistry, MetricType};
use instance_monitor::error::AppError;

// Helper function to get the IP address
fn get_instance_ip() -> Option<String> {
    use std::net::ToSocketAddrs;
    
    let hostname = hostname::get().ok()?.into_string().ok()?;
    let addresses = (hostname.as_str(), 0).to_socket_addrs().ok()?;
    addresses.filter(|addr| addr.is_ipv4()).next().map(|addr| addr.ip().to_string())
}

// System monitor for CPU and memory usage
pub fn system_monitor(registry: MetricRegistry) -> Result<(), AppError> {
    registry.register_metric("cpu_usage", MetricType::Gauge)?;
    registry.register_metric("memory_usage", MetricType::Gauge)?;

    // Get the instance IP address to use as a label
    let instance_ip = get_instance_ip().unwrap_or_else(|| "unknown".to_string());

    // Add labels with instance IP
    if let Some(mut metric) = registry.get_metric("cpu_usage") {
        metric.add_label("instance", &instance_ip);
    }
    if let Some(mut metric) = registry.get_metric("memory_usage") {
        metric.add_label("instance", &instance_ip);
    }

    thread::spawn(move || {
        let mut system = System::new_all();

        loop {
            system.refresh_all();

            // Get CPU usage
            let cpu_usage = system.global_cpu_info().cpu_usage();

            // Get memory usage
            let total_memory = system.total_memory();
            let used_memory = system.used_memory();
            let memory_usage_percentage = (used_memory as f64 / total_memory as f64) * 100.0;

            // Update metrics
            {
                if let Err(e) = registry.update_metric("cpu_usage", cpu_usage.into()) {
                    log::error!("Error updating cpu_usage: {}", e);
                }
                if let Err(e) = registry.update_metric("memory_usage", memory_usage_percentage) {
                    log::error!("Error updating memory_usage: {}", e);
                }
            }

            thread::sleep(Duration::from_secs(5));
        }
    });

    Ok(())
}
