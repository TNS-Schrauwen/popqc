//! Outlier detection algorithms

#![allow(clippy::cast_precision_loss)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutlierMethod {
    Iqr { factor: f64 },
    Zscore { threshold: f64 },
    Mad { factor: f64 },
}

impl Default for OutlierMethod {
    fn default() -> Self {
        Self::Iqr { factor: 1.5 }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OutlierResult {
    pub sample: String,
    pub metric: String,
    pub value: f64,
    pub is_outlier: bool,
    pub direction: OutlierDirection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutlierDirection {
    Low,
    High,
    None,
}

#[must_use]
pub fn detect_outliers(values: &[f64], method: OutlierMethod) -> Vec<bool> {
    match method {
        OutlierMethod::Iqr { factor } => detect_iqr(values, factor),
        OutlierMethod::Zscore { threshold } => detect_zscore(values, threshold),
        OutlierMethod::Mad { factor } => detect_mad(values, factor),
    }
}

fn detect_iqr(values: &[f64], factor: f64) -> Vec<bool> {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    if n < 4 {
        return vec![false; values.len()];
    }

    let q1 = sorted[n / 4];
    let q3 = sorted[3 * n / 4];
    let iqr = q3 - q1;
    let lower = q1 - factor * iqr;
    let upper = q3 + factor * iqr;

    values
        .iter()
        .map(|&v| v.is_finite() && (v < lower || v > upper))
        .collect()
}

fn detect_zscore(values: &[f64], threshold: f64) -> Vec<bool> {
    let valid: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let n = valid.len();
    if n < 3 {
        return vec![false; values.len()];
    }

    let mean = valid.iter().sum::<f64>() / n as f64;
    let variance = valid.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return vec![false; values.len()];
    }

    values
        .iter()
        .map(|&v| v.is_finite() && ((v - mean) / std_dev).abs() > threshold)
        .collect()
}

fn detect_mad(values: &[f64], factor: f64) -> Vec<bool> {
    let mut valid: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    valid.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = valid.len();
    if n < 3 {
        return vec![false; values.len()];
    }

    let median = valid[n / 2];
    let mut deviations: Vec<f64> = valid.iter().map(|v| (v - median).abs()).collect();
    deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mad = deviations[n / 2] * 1.4826;

    if mad == 0.0 {
        return vec![false; values.len()];
    }

    let threshold = factor * mad;
    values
        .iter()
        .map(|&v| v.is_finite() && (v - median).abs() > threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iqr_detection() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let outliers = detect_outliers(&values, OutlierMethod::Iqr { factor: 1.5 });
        assert!(!outliers[0]);
        assert!(outliers[5]);
    }

    #[test]
    fn test_zscore_detection() {
        let values = vec![10.0, 10.1, 9.9, 10.2, 9.8, 50.0];
        let outliers = detect_outliers(&values, OutlierMethod::Zscore { threshold: 2.0 });
        assert!(outliers[5]);
    }
}
