//! Core data model

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleRecord {
    pub sample: String,
    pub metadata: HashMap<String, String>,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CohortFrame {
    records: Vec<SampleRecord>,
    sample_index: HashMap<String, usize>,
    metric_ids: Vec<String>,
    metadata_keys: Vec<String>,
}

impl CohortFrame {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn num_samples(&self) -> usize {
        self.records.len()
    }

    #[must_use]
    pub fn num_metrics(&self) -> usize {
        self.metric_ids.len()
    }

    #[must_use]
    pub fn num_metadata_fields(&self) -> usize {
        self.metadata_keys.len()
    }

    #[must_use]
    pub fn sample_names(&self) -> Vec<&str> {
        self.records.iter().map(|r| r.sample.as_str()).collect()
    }

    #[must_use]
    pub fn metric_ids(&self) -> &[String] {
        &self.metric_ids
    }

    #[must_use]
    pub fn metadata_keys(&self) -> &[String] {
        &self.metadata_keys
    }

    #[must_use]
    pub fn get_sample(&self, name: &str) -> Option<&SampleRecord> {
        self.sample_index.get(name).map(|&idx| &self.records[idx])
    }

    #[must_use]
    pub fn metric_values(&self, metric_id: &str) -> Vec<Option<f64>> {
        self.records
            .iter()
            .map(|r| r.metrics.get(metric_id).copied())
            .collect()
    }

    #[must_use]
    pub fn metric_values_non_null(&self, metric_id: &str) -> Vec<f64> {
        self.records
            .iter()
            .filter_map(|r| r.metrics.get(metric_id).copied())
            .collect()
    }

    pub fn upsert_sample(&mut self, record: SampleRecord) {
        for key in record.metrics.keys() {
            if !self.metric_ids.contains(key) {
                self.metric_ids.push(key.clone());
            }
        }

        for key in record.metadata.keys() {
            if !self.metadata_keys.contains(key) {
                self.metadata_keys.push(key.clone());
                self.metadata_keys.sort();
            }
        }

        if let Some(&idx) = self.sample_index.get(&record.sample) {
            let existing = &mut self.records[idx];
            for (k, v) in &record.metrics {
                existing.metrics.insert(k.clone(), *v);
            }
            for (k, v) in &record.metadata {
                if !v.is_empty() {
                    existing.metadata.insert(k.clone(), v.clone());
                }
            }
        } else {
            let idx = self.records.len();
            self.sample_index.insert(record.sample.clone(), idx);
            self.records.push(record);
        }
    }

    #[must_use]
    pub fn records(&self) -> &[SampleRecord] {
        &self.records
    }
    #[must_use]
    pub fn to_json_records(&self) -> Vec<serde_json::Value> {
        self.records
            .iter()
            .map(|r| {
                let mut map = serde_json::Map::new();
                map.insert(
                    "Sample".to_string(),
                    serde_json::Value::String(r.sample.clone()),
                );

                for key in &self.metadata_keys {
                    let val = r.metadata.get(key).cloned().unwrap_or_default();
                    map.insert(key.clone(), serde_json::Value::String(val));
                }

                for metric_id in &self.metric_ids {
                    if let Some(&v) = r.metrics.get(metric_id) {
                        if v.is_finite() {
                            let rounded = (v * 100.0).round() / 100.0;
                            if let Some(n) = serde_json::Number::from_f64(rounded) {
                                map.insert(metric_id.clone(), serde_json::Value::Number(n));
                            } else {
                                map.insert(
                                    metric_id.clone(),
                                    serde_json::Value::String(String::new()),
                                );
                            }
                        } else {
                            map.insert(metric_id.clone(), serde_json::Value::String(String::new()));
                        }
                    } else {
                        map.insert(metric_id.clone(), serde_json::Value::String(String::new()));
                    }
                }

                serde_json::Value::Object(map)
            })
            .collect()
    }
}
