use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Note: Global metrics removed - each client instance has its own metrics
// This prevents cross-client metric pollution

/// Metrics tracks SDK performance and usage statistics
#[derive(Clone)]
pub struct Metrics {
    total_transactions: Arc<AtomicU64>,
    successful_transactions: Arc<AtomicU64>,
    failed_transactions: Arc<AtomicU64>,
    total_fee_paid: Arc<Mutex<f64>>,
    total_latency_ms: Arc<AtomicU64>, // Stored in milliseconds
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Returns the global metrics instance
    pub fn new() -> Self {
        Self {
            total_transactions: Arc::new(AtomicU64::new(0)),
            successful_transactions: Arc::new(AtomicU64::new(0)),
            failed_transactions: Arc::new(AtomicU64::new(0)),
            total_fee_paid: Arc::new(Mutex::new(0.0)),
            total_latency_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Records a transaction attempt
    pub fn record_transaction(&self, success: bool, fee: f64, latency: Duration) {
        self.total_transactions.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.successful_transactions.fetch_add(1, Ordering::Relaxed);
            if let Ok(mut total_fee) = self.total_fee_paid.lock() {
                *total_fee += fee;
            }
        } else {
            self.failed_transactions.fetch_add(1, Ordering::Relaxed);
        }

        // Accumulate total latency (average calculated in get_stats)
        let latency_ms = latency.as_millis() as u64;
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }

    /// Returns current statistics
    pub fn get_stats(&self) -> std::collections::HashMap<String, f64> {
        let total = self.total_transactions.load(Ordering::Relaxed) as f64;
        let successful = self.successful_transactions.load(Ordering::Relaxed) as f64;
        let failed = self.failed_transactions.load(Ordering::Relaxed) as f64;
        let total_fee = self.total_fee_paid.lock().map(|f| *f).unwrap_or(0.0);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed) as f64;

        let mut stats = std::collections::HashMap::new();
        stats.insert("total_transactions".to_string(), total);
        stats.insert("successful_transactions".to_string(), successful);
        stats.insert("failed_transactions".to_string(), failed);
        stats.insert("total_fee_paid".to_string(), total_fee);
        
        if total > 0.0 {
            stats.insert("average_latency_ms".to_string(), total_latency / total);
            stats.insert("success_rate".to_string(), successful / total);
        } else {
            stats.insert("average_latency_ms".to_string(), 0.0);
            stats.insert("success_rate".to_string(), 0.0);
        }

        stats
    }

    /// Clears all metrics (useful for testing)
    pub fn reset(&self) {
        self.total_transactions.store(0, Ordering::Relaxed);
        self.successful_transactions.store(0, Ordering::Relaxed);
        self.failed_transactions.store(0, Ordering::Relaxed);
        if let Ok(mut total_fee) = self.total_fee_paid.lock() {
            *total_fee = 0.0;
        }
        self.total_latency_ms.store(0, Ordering::Relaxed);
    }
}

// Note: Global metrics removed - each client instance has its own metrics
// This prevents cross-client metric pollution

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new();
        let stats = metrics.get_stats();
        assert_eq!(stats["total_transactions"], 0.0);
        assert_eq!(stats["successful_transactions"], 0.0);
        assert_eq!(stats["failed_transactions"], 0.0);
    }

    #[test]
    fn test_metrics_record_success() {
        let metrics = Metrics::new();
        metrics.record_transaction(true, 0.05, Duration::from_millis(100));
        let stats = metrics.get_stats();
        assert_eq!(stats["total_transactions"], 1.0);
        assert_eq!(stats["successful_transactions"], 1.0);
        assert_eq!(stats["failed_transactions"], 0.0);
        assert_eq!(stats["total_fee_paid"], 0.05);
        assert_eq!(stats["success_rate"], 1.0);
    }

    #[test]
    fn test_metrics_record_failure() {
        let metrics = Metrics::new();
        metrics.record_transaction(false, 0.0, Duration::from_millis(50));
        let stats = metrics.get_stats();
        assert_eq!(stats["total_transactions"], 1.0);
        assert_eq!(stats["successful_transactions"], 0.0);
        assert_eq!(stats["failed_transactions"], 1.0);
        assert_eq!(stats["success_rate"], 0.0);
    }

    #[test]
    fn test_metrics_multiple_transactions() {
        let metrics = Metrics::new();
        metrics.record_transaction(true, 0.05, Duration::from_millis(100));
        metrics.record_transaction(true, 0.03, Duration::from_millis(80));
        metrics.record_transaction(false, 0.0, Duration::from_millis(50));
        
        let stats = metrics.get_stats();
        assert_eq!(stats["total_transactions"], 3.0);
        assert_eq!(stats["successful_transactions"], 2.0);
        assert_eq!(stats["failed_transactions"], 1.0);
        assert_eq!(stats["total_fee_paid"], 0.08);
        assert!((stats["success_rate"] - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = Metrics::new();
        metrics.record_transaction(true, 0.05, Duration::from_millis(100));
        metrics.reset();
        let stats = metrics.get_stats();
        assert_eq!(stats["total_transactions"], 0.0);
        assert_eq!(stats["successful_transactions"], 0.0);
    }

    #[test]
    fn test_metrics_average_latency() {
        let metrics = Metrics::new();
        metrics.record_transaction(true, 0.05, Duration::from_millis(100));
        metrics.record_transaction(true, 0.03, Duration::from_millis(200));
        let stats = metrics.get_stats();
        assert_eq!(stats["average_latency_ms"], 150.0);
    }
}
