//! Metric definitions, thresholds, and catalog

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    Float,
    Integer,
    Percent,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThresholdLevel {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub warn_low: Option<f64>,
    pub warn_high: Option<f64>,
    pub fail_low: Option<f64>,
    pub fail_high: Option<f64>,
}

impl Thresholds {
    #[must_use]
    pub fn evaluate(&self, value: f64) -> ThresholdLevel {
    if let Some(fl) = self.fail_low && value < fl {
        return ThresholdLevel::Fail;
    }

    if let Some(fh) = self.fail_high && value > fh {
        return ThresholdLevel::Fail;
    }

    if let Some(wl) = self.warn_low && value < wl {
        return ThresholdLevel::Warn;
    }

    if let Some(wh) = self.warn_high && value > wh {
        return ThresholdLevel::Warn;
    }

    ThresholdLevel::Pass
}

    #[must_use]
    pub fn from_iqr(values: &[f64], factor: f64) -> Self {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted.len();
        if n < 4 {
            return Self {
                warn_low: None,
                warn_high: None,
                fail_low: None,
                fail_high: None,
            };
        }

        let q1 = sorted[n / 4];
        let q3 = sorted[3 * n / 4];
        let iqr = q3 - q1;

        Self {
            warn_low: Some(q1 - factor * iqr),
            warn_high: Some(q3 + factor * iqr),
            fail_low: Some(q1 - 2.0 * factor * iqr),
            fail_high: Some(q3 + 2.0 * factor * iqr),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: String,
    pub label: String,
    pub description: String,
    pub metric_type: MetricType,
    pub source_tool: String,
    pub thresholds: Option<Thresholds>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricCatalog {
    metrics: HashMap<String, Metric>,
}

impl MetricCatalog {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    pub fn register(&mut self, metric: Metric) {
        self.metrics.insert(metric.id.clone(), metric);
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Metric> {
        self.metrics.get(id)
    }

    #[must_use]
    pub fn metric_ids(&self) -> Vec<&str> {
        self.metrics.keys().map(String::as_str).collect()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }
}
