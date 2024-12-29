use dashmap::DashMap;
use instance_monitor::error::AppError;
use std::sync::Arc;
use std::time::Instant;

// A Metric type for representing different types of metrics
#[derive(Debug, Clone)]
pub enum MetricType {
    Gauge,
}

// A Single Metric
#[derive(Debug, Clone)]
pub struct Metric {
    name: String,
    metric_type: MetricType,
    value: f64,
    labels: Vec<(String, String)>,
    timestamp: Instant,
}

impl Metric {
    pub fn new(name: &str, metric_type: MetricType) -> Self {
        Self {
            name: name.to_string(),
            metric_type,
            value: 0.0,
            labels: Vec::new(),
            timestamp: Instant::now(),
        }
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
        self.timestamp = Instant::now();
    }

    pub fn add_label(&mut self, key: &str, value: &str) {
        self.labels.push((key.to_string(), value.to_string()));
    }

    pub fn to_prometheus_format(&self) -> String {
        let labels = self
            .labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(",");
        let label_str = if labels.is_empty() {
            String::new()
        } else {
            format!("{{{}}}", labels)
        };

        format!("{}{} {}", self.name, label_str, self.value)
    }
}

// A Metric Registry to manage and export metrics
#[derive(Debug, Clone, Default)]
pub struct MetricRegistry {
    metrics: Arc<DashMap<String, Metric>>,
}

impl MetricRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_metric(
        &self,
        name: &str,
    ) -> Option<dashmap::mapref::one::RefMut<'_, String, Metric>> {
        self.metrics.get_mut(name)
    }

    pub fn register_metric(&self, name: &str, metric_type: MetricType) -> Result<(), AppError> {
        let metrics = self.metrics.clone();
        metrics
            .entry(name.to_string())
            .or_insert(Metric::new(name, metric_type));
        Ok(())
    }

    pub fn update_metric(&self, name: &str, value: f64) -> Result<(), AppError> {
        if let Some(mut metric) = self.get_metric(name) {
            match metric.metric_type {
                MetricType::Gauge => metric.set_value(value),
            }
        }
        Ok(())
    }

    pub fn get_prometheus_metrics(&self) -> Result<String, AppError> {
        let metrics = self
            .metrics
            .iter()
            .map(|entry| entry.to_prometheus_format())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(format!("{metrics}\n"))
    }
}
