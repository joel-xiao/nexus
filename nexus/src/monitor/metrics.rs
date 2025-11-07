use dashmap::DashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct Metrics {
    counters: Arc<DashMap<String, u64>>,
    gauges: Arc<DashMap<String, f64>>,
    histograms: Arc<DashMap<String, Vec<f64>>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(DashMap::new()),
            gauges: Arc::new(DashMap::new()),
            histograms: Arc::new(DashMap::new()),
        }
    }

    pub fn increment(&self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn increment_by(&self, name: &str, value: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += value;
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn record_histogram(&self, name: &str, value: f64) {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
        
        // 限制直方图大小
        if let Some(mut vec) = self.histograms.get_mut(name) {
            if vec.len() > 1000 {
                vec.remove(0);
            }
        }
    }

    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.get(name).map(|v| *v.value()).unwrap_or(0)
    }

    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).map(|v| *v.value())
    }

    pub fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        self.histograms.get(name).map(|values| {
            let v: Vec<f64> = values.iter().copied().collect();
            let count = v.len();
            if count == 0 {
                return HistogramStats {
                    count: 0,
                    min: 0.0,
                    max: 0.0,
                    avg: 0.0,
                    p50: 0.0,
                    p95: 0.0,
                    p99: 0.0,
                };
            }
            
            let mut sorted = v.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let min = sorted[0];
            let max = sorted[count - 1];
            let sum: f64 = v.iter().sum();
            let avg = sum / count as f64;
            let p50 = sorted[count / 2];
            let p95 = sorted[(count as f64 * 0.95) as usize];
            let p99 = sorted[(count as f64 * 0.99) as usize];
            
            HistogramStats {
                count,
                min,
                max,
                avg,
                p50,
                p95,
                p99,
            }
        })
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            counters: self.counters.iter().map(|e| (e.key().clone(), *e.value())).collect(),
            gauges: self.gauges.iter().map(|e| (e.key().clone(), *e.value())).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HistogramStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsSnapshot {
    pub counters: std::collections::HashMap<String, u64>,
    pub gauges: std::collections::HashMap<String, f64>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

