use crate::utils::error::ErpResult;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: DateTime<Utc>,
    pub endpoint: String,
    pub method: String,
    pub response_time_ms: u64,
    pub status_code: u16,
    pub request_size_bytes: u64,
    pub response_size_bytes: u64,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub timestamp: DateTime<Utc>,
    pub endpoint: String,
    pub method: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub p50_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformance {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub memory_usage_percent: f32,
    pub heap_size_mb: u64,
    pub garbage_collections: u32,
    pub thread_count: u32,
    pub active_connections: u32,
    pub connection_pool_usage: f32,
    pub database_connections: u32,
    pub cache_hit_ratio: f32,
    pub disk_io_read_mb: u64,
    pub disk_io_write_mb: u64,
    pub network_io_rx_mb: u64,
    pub network_io_tx_mb: u64,
}

#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub enabled: bool,
    pub sample_rate: f64,
    pub buffer_size: usize,
    pub aggregation_window_minutes: i64,
    pub retention_hours: i64,
    pub slow_query_threshold_ms: u64,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub response_time_warning_ms: u64,
    pub response_time_critical_ms: u64,
    pub error_rate_warning_percent: f64,
    pub error_rate_critical_percent: f64,
    pub memory_usage_warning_percent: f32,
    pub memory_usage_critical_percent: f32,
    pub cpu_usage_warning_percent: f32,
    pub cpu_usage_critical_percent: f32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate: 1.0,
            buffer_size: 10000,
            aggregation_window_minutes: 5,
            retention_hours: 24,
            slow_query_threshold_ms: 1000,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            response_time_warning_ms: 1000,
            response_time_critical_ms: 3000,
            error_rate_warning_percent: 5.0,
            error_rate_critical_percent: 10.0,
            memory_usage_warning_percent: 80.0,
            memory_usage_critical_percent: 90.0,
            cpu_usage_warning_percent: 70.0,
            cpu_usage_critical_percent: 85.0,
        }
    }
}

#[async_trait::async_trait]
pub trait PerformanceRepository: Send + Sync {
    async fn store_metrics(&self, metrics: &[PerformanceMetrics]) -> ErpResult<()>;
    async fn store_aggregated_metrics(&self, metrics: &AggregatedMetrics) -> ErpResult<()>;
    async fn store_system_performance(&self, performance: &SystemPerformance) -> ErpResult<()>;

    async fn get_metrics(
        &self,
        endpoint: Option<&str>,
        since: DateTime<Utc>,
        limit: Option<usize>,
    ) -> ErpResult<Vec<PerformanceMetrics>>;

    async fn get_aggregated_metrics(
        &self,
        endpoint: Option<&str>,
        since: DateTime<Utc>,
    ) -> ErpResult<Vec<AggregatedMetrics>>;

    async fn get_system_performance(
        &self,
        since: DateTime<Utc>,
    ) -> ErpResult<Vec<SystemPerformance>>;

    async fn cleanup_old_metrics(&self, before: DateTime<Utc>) -> ErpResult<u64>;
}

pub struct PerformanceMonitor {
    config: PerformanceConfig,
    repository: Arc<dyn PerformanceRepository>,
    metrics_buffer: Arc<Mutex<VecDeque<PerformanceMetrics>>>,
    active_requests: Arc<RwLock<HashMap<String, RequestTracker>>>,
    aggregated_cache: Arc<Mutex<HashMap<String, MetricsAccumulator>>>,
    last_aggregation: Arc<Mutex<DateTime<Utc>>>,
}

#[derive(Debug, Clone)]
struct RequestTracker {
    start_time: Instant,
    endpoint: String,
    method: String,
    user_id: Option<String>,
    session_id: Option<String>,
}

#[derive(Debug, Clone)]
struct MetricsAccumulator {
    endpoint: String,
    method: String,
    response_times: Vec<u64>,
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_bytes_sent: u64,
    total_bytes_received: u64,
    _window_start: DateTime<Utc>,
}

impl PerformanceMonitor {
    pub fn new(config: PerformanceConfig, repository: Arc<dyn PerformanceRepository>) -> Self {
        Self {
            config,
            repository,
            metrics_buffer: Arc::new(Mutex::new(VecDeque::new())),
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            aggregated_cache: Arc::new(Mutex::new(HashMap::new())),
            last_aggregation: Arc::new(Mutex::new(Utc::now())),
        }
    }

    pub async fn start_request(
        &self,
        request_id: String,
        endpoint: String,
        method: String,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> ErpResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let tracker = RequestTracker {
            start_time: Instant::now(),
            endpoint,
            method,
            user_id,
            session_id,
        };

        let mut active_requests = self.active_requests.write().await;
        active_requests.insert(request_id, tracker);

        Ok(())
    }

    pub async fn end_request(
        &self,
        request_id: String,
        status_code: u16,
        request_size: u64,
        response_size: u64,
        error_message: Option<String>,
    ) -> ErpResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let tracker = {
            let mut active_requests = self.active_requests.write().await;
            active_requests.remove(&request_id)
        };

        if let Some(tracker) = tracker {
            let response_time_ms = tracker.start_time.elapsed().as_millis() as u64;

            // Sample based on configuration
            if self.should_sample() {
                let metrics = PerformanceMetrics {
                    timestamp: Utc::now(),
                    endpoint: tracker.endpoint.clone(),
                    method: tracker.method.clone(),
                    response_time_ms,
                    status_code,
                    request_size_bytes: request_size,
                    response_size_bytes: response_size,
                    user_id: tracker.user_id.clone(),
                    session_id: tracker.session_id.clone(),
                    error_message,
                };

                self.record_metrics(metrics).await?;
            }

            // Update aggregated metrics
            self.update_aggregated_metrics(
                &tracker,
                response_time_ms,
                status_code,
                request_size,
                response_size,
            )
            .await?;
        }

        Ok(())
    }

    pub async fn record_system_performance(&self, performance: SystemPerformance) -> ErpResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        self.repository
            .store_system_performance(&performance)
            .await?;
        debug!("System performance metrics stored");

        // Check for alerts
        self.check_system_performance_alerts(&performance).await?;

        Ok(())
    }

    pub async fn get_endpoint_performance(
        &self,
        endpoint: &str,
        hours: i64,
    ) -> ErpResult<EndpointPerformanceSummary> {
        let since = Utc::now() - Duration::hours(hours);
        let metrics = self
            .repository
            .get_metrics(Some(endpoint), since, None)
            .await?;

        if metrics.is_empty() {
            return Ok(EndpointPerformanceSummary {
                endpoint: endpoint.to_string(),
                period_hours: hours,
                total_requests: 0,
                avg_response_time_ms: 0.0,
                min_response_time_ms: 0,
                max_response_time_ms: 0,
                success_rate: 0.0,
                requests_per_hour: 0.0,
                error_count: 0,
                slow_requests: 0,
            });
        }

        let response_times: Vec<u64> = metrics.iter().map(|m| m.response_time_ms).collect();
        let successful = metrics.iter().filter(|m| m.status_code < 400).count();
        let errors = metrics.len() - successful;
        let slow_requests = metrics
            .iter()
            .filter(|m| m.response_time_ms > self.config.slow_query_threshold_ms)
            .count();

        Ok(EndpointPerformanceSummary {
            endpoint: endpoint.to_string(),
            period_hours: hours,
            total_requests: metrics.len(),
            avg_response_time_ms: response_times.iter().sum::<u64>() as f64
                / response_times.len() as f64,
            min_response_time_ms: *response_times.iter().min().unwrap_or(&0),
            max_response_time_ms: *response_times.iter().max().unwrap_or(&0),
            success_rate: (successful as f64 / metrics.len() as f64) * 100.0,
            requests_per_hour: metrics.len() as f64 / hours as f64,
            error_count: errors,
            slow_requests,
        })
    }

    pub async fn get_performance_trends(&self, hours: i64) -> ErpResult<PerformanceTrends> {
        let since = Utc::now() - Duration::hours(hours);
        let aggregated = self.repository.get_aggregated_metrics(None, since).await?;

        let mut trends = PerformanceTrends {
            period_hours: hours,
            endpoints: HashMap::new(),
            overall_avg_response_time: 0.0,
            overall_error_rate: 0.0,
            total_requests: 0,
            trending_up: Vec::new(),
            trending_down: Vec::new(),
        };

        if aggregated.is_empty() {
            return Ok(trends);
        }

        let mut endpoint_stats: HashMap<String, Vec<&AggregatedMetrics>> = HashMap::new();
        for metric in &aggregated {
            endpoint_stats
                .entry(metric.endpoint.clone())
                .or_default()
                .push(metric);
        }

        let mut total_requests = 0u64;
        let mut total_response_time = 0.0;
        let mut total_errors = 0u64;

        for (endpoint, metrics) in endpoint_stats {
            let recent_metrics = metrics.iter().rev().take(6).collect::<Vec<_>>(); // Last 6 windows
            let older_metrics = metrics.iter().rev().skip(6).take(6).collect::<Vec<_>>();

            let recent_avg = if !recent_metrics.is_empty() {
                recent_metrics
                    .iter()
                    .map(|m| m.avg_response_time_ms)
                    .sum::<f64>()
                    / recent_metrics.len() as f64
            } else {
                0.0
            };

            let older_avg = if !older_metrics.is_empty() {
                older_metrics
                    .iter()
                    .map(|m| m.avg_response_time_ms)
                    .sum::<f64>()
                    / older_metrics.len() as f64
            } else {
                recent_avg
            };

            let trend = if older_avg > 0.0 {
                ((recent_avg - older_avg) / older_avg) * 100.0
            } else {
                0.0
            };

            let endpoint_total_requests: u64 = metrics.iter().map(|m| m.total_requests).sum();
            let endpoint_total_errors: u64 = metrics.iter().map(|m| m.failed_requests).sum();
            let endpoint_avg_response: f64 = metrics
                .iter()
                .map(|m| m.avg_response_time_ms * m.total_requests as f64)
                .sum::<f64>()
                / endpoint_total_requests as f64;

            total_requests += endpoint_total_requests;
            total_response_time += endpoint_avg_response * endpoint_total_requests as f64;
            total_errors += endpoint_total_errors;

            trends.endpoints.insert(
                endpoint.clone(),
                EndpointTrend {
                    avg_response_time: endpoint_avg_response,
                    error_rate: (endpoint_total_errors as f64 / endpoint_total_requests as f64)
                        * 100.0,
                    trend_percentage: trend,
                    total_requests: endpoint_total_requests,
                },
            );

            if trend > 10.0 {
                trends.trending_up.push(endpoint.clone());
            } else if trend < -10.0 {
                trends.trending_down.push(endpoint.clone());
            }
        }

        if total_requests > 0 {
            trends.overall_avg_response_time = total_response_time / total_requests as f64;
            trends.overall_error_rate = (total_errors as f64 / total_requests as f64) * 100.0;
        }

        trends.total_requests = total_requests;

        Ok(trends)
    }

    pub async fn flush_metrics(&self) -> ErpResult<()> {
        let metrics = {
            let mut buffer = self.metrics_buffer.lock().unwrap();
            let metrics: Vec<PerformanceMetrics> = buffer.drain(..).collect();
            metrics
        };

        if !metrics.is_empty() {
            self.repository.store_metrics(&metrics).await?;
            debug!(
                "Flushed {} performance metrics to repository",
                metrics.len()
            );
        }

        // Aggregate and flush accumulated metrics
        self.aggregate_and_flush().await?;

        Ok(())
    }

    pub async fn cleanup_old_metrics(&self) -> ErpResult<u64> {
        let cutoff = Utc::now() - Duration::hours(self.config.retention_hours);
        let removed = self.repository.cleanup_old_metrics(cutoff).await?;

        if removed > 0 {
            info!("Cleaned up {} old performance metrics", removed);
        }

        Ok(removed)
    }

    async fn record_metrics(&self, metrics: PerformanceMetrics) -> ErpResult<()> {
        let mut buffer = self.metrics_buffer.lock().unwrap();
        buffer.push_back(metrics);

        if buffer.len() >= self.config.buffer_size {
            // Buffer is full, we should flush asynchronously
            warn!("Performance metrics buffer is full, consider increasing buffer size or flush frequency");
        }

        Ok(())
    }

    async fn update_aggregated_metrics(
        &self,
        tracker: &RequestTracker,
        response_time: u64,
        status_code: u16,
        request_size: u64,
        response_size: u64,
    ) -> ErpResult<()> {
        let key = format!("{}:{}", tracker.method, tracker.endpoint);
        let mut cache = self.aggregated_cache.lock().unwrap();

        let accumulator = cache.entry(key).or_insert_with(|| MetricsAccumulator {
            endpoint: tracker.endpoint.clone(),
            method: tracker.method.clone(),
            response_times: Vec::new(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            _window_start: Utc::now(),
        });

        accumulator.response_times.push(response_time);
        accumulator.total_requests += 1;
        accumulator.total_bytes_sent += request_size;
        accumulator.total_bytes_received += response_size;

        if status_code < 400 {
            accumulator.successful_requests += 1;
        } else {
            accumulator.failed_requests += 1;
        }

        Ok(())
    }

    async fn aggregate_and_flush(&self) -> ErpResult<()> {
        let should_aggregate = {
            let last_aggregation = self.last_aggregation.lock().unwrap();
            let window_duration = Duration::minutes(self.config.aggregation_window_minutes);
            Utc::now().signed_duration_since(*last_aggregation) >= window_duration
        };

        if !should_aggregate {
            return Ok(());
        }

        let accumulators = {
            let mut cache = self.aggregated_cache.lock().unwrap();
            let accumulators: Vec<MetricsAccumulator> = cache.drain().map(|(_, acc)| acc).collect();
            accumulators
        };

        for accumulator in accumulators {
            if accumulator.total_requests > 0 {
                let aggregated = self.create_aggregated_metrics(accumulator)?;
                self.repository
                    .store_aggregated_metrics(&aggregated)
                    .await?;
            }
        }

        *self.last_aggregation.lock().unwrap() = Utc::now();

        Ok(())
    }

    fn create_aggregated_metrics(
        &self,
        accumulator: MetricsAccumulator,
    ) -> ErpResult<AggregatedMetrics> {
        let mut response_times = accumulator.response_times;
        response_times.sort_unstable();

        let total_response_time: u64 = response_times.iter().sum();
        let avg_response_time = total_response_time as f64 / response_times.len() as f64;

        let p50 = self.percentile(&response_times, 0.5);
        let p95 = self.percentile(&response_times, 0.95);
        let p99 = self.percentile(&response_times, 0.99);

        let window_duration_seconds = self.config.aggregation_window_minutes * 60;
        let requests_per_second =
            accumulator.total_requests as f64 / window_duration_seconds as f64;

        let error_rate =
            (accumulator.failed_requests as f64 / accumulator.total_requests as f64) * 100.0;

        Ok(AggregatedMetrics {
            timestamp: Utc::now(),
            endpoint: accumulator.endpoint,
            method: accumulator.method,
            total_requests: accumulator.total_requests,
            successful_requests: accumulator.successful_requests,
            failed_requests: accumulator.failed_requests,
            avg_response_time_ms: avg_response_time,
            min_response_time_ms: *response_times.first().unwrap_or(&0),
            max_response_time_ms: *response_times.last().unwrap_or(&0),
            p50_response_time_ms: p50,
            p95_response_time_ms: p95,
            p99_response_time_ms: p99,
            requests_per_second,
            error_rate,
            total_bytes_sent: accumulator.total_bytes_sent,
            total_bytes_received: accumulator.total_bytes_received,
        })
    }

    fn percentile(&self, sorted_values: &[u64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (percentile * (sorted_values.len() - 1) as f64).round() as usize;
        sorted_values[index.min(sorted_values.len() - 1)] as f64
    }

    fn should_sample(&self) -> bool {
        if self.config.sample_rate >= 1.0 {
            true
        } else {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            rng.gen::<f64>() < self.config.sample_rate
        }
    }

    async fn check_system_performance_alerts(
        &self,
        performance: &SystemPerformance,
    ) -> ErpResult<()> {
        let thresholds = &self.config.alert_thresholds;

        if performance.cpu_usage_percent > thresholds.cpu_usage_critical_percent {
            warn!(
                "CRITICAL: CPU usage is {:.1}%",
                performance.cpu_usage_percent
            );
        } else if performance.cpu_usage_percent > thresholds.cpu_usage_warning_percent {
            warn!(
                "WARNING: CPU usage is {:.1}%",
                performance.cpu_usage_percent
            );
        }

        if performance.memory_usage_percent > thresholds.memory_usage_critical_percent {
            warn!(
                "CRITICAL: Memory usage is {:.1}%",
                performance.memory_usage_percent
            );
        } else if performance.memory_usage_percent > thresholds.memory_usage_warning_percent {
            warn!(
                "WARNING: Memory usage is {:.1}%",
                performance.memory_usage_percent
            );
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPerformanceSummary {
    pub endpoint: String,
    pub period_hours: i64,
    pub total_requests: usize,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub success_rate: f64,
    pub requests_per_hour: f64,
    pub error_count: usize,
    pub slow_requests: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub period_hours: i64,
    pub endpoints: HashMap<String, EndpointTrend>,
    pub overall_avg_response_time: f64,
    pub overall_error_rate: f64,
    pub total_requests: u64,
    pub trending_up: Vec<String>,
    pub trending_down: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointTrend {
    pub avg_response_time: f64,
    pub error_rate: f64,
    pub trend_percentage: f64,
    pub total_requests: u64,
}

// Mock implementation for testing
#[derive(Debug)]
pub struct MockPerformanceRepository {
    metrics: Arc<Mutex<Vec<PerformanceMetrics>>>,
    aggregated: Arc<Mutex<Vec<AggregatedMetrics>>>,
    system_performance: Arc<Mutex<Vec<SystemPerformance>>>,
}

impl MockPerformanceRepository {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            aggregated: Arc::new(Mutex::new(Vec::new())),
            system_performance: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl PerformanceRepository for MockPerformanceRepository {
    async fn store_metrics(&self, metrics: &[PerformanceMetrics]) -> ErpResult<()> {
        let mut stored_metrics = self.metrics.lock().unwrap();
        stored_metrics.extend_from_slice(metrics);
        Ok(())
    }

    async fn store_aggregated_metrics(&self, metrics: &AggregatedMetrics) -> ErpResult<()> {
        let mut stored_metrics = self.aggregated.lock().unwrap();
        stored_metrics.push(metrics.clone());
        Ok(())
    }

    async fn store_system_performance(&self, performance: &SystemPerformance) -> ErpResult<()> {
        let mut stored_performance = self.system_performance.lock().unwrap();
        stored_performance.push(performance.clone());
        Ok(())
    }

    async fn get_metrics(
        &self,
        endpoint: Option<&str>,
        since: DateTime<Utc>,
        limit: Option<usize>,
    ) -> ErpResult<Vec<PerformanceMetrics>> {
        let metrics = self.metrics.lock().unwrap();
        let mut filtered: Vec<PerformanceMetrics> = metrics
            .iter()
            .filter(|m| m.timestamp > since && endpoint.map_or(true, |e| m.endpoint == e))
            .cloned()
            .collect();

        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        Ok(filtered)
    }

    async fn get_aggregated_metrics(
        &self,
        endpoint: Option<&str>,
        since: DateTime<Utc>,
    ) -> ErpResult<Vec<AggregatedMetrics>> {
        let metrics = self.aggregated.lock().unwrap();
        Ok(metrics
            .iter()
            .filter(|m| m.timestamp > since && endpoint.map_or(true, |e| m.endpoint == e))
            .cloned()
            .collect())
    }

    async fn get_system_performance(
        &self,
        since: DateTime<Utc>,
    ) -> ErpResult<Vec<SystemPerformance>> {
        let performance = self.system_performance.lock().unwrap();
        Ok(performance
            .iter()
            .filter(|p| p.timestamp > since)
            .cloned()
            .collect())
    }

    async fn cleanup_old_metrics(&self, before: DateTime<Utc>) -> ErpResult<u64> {
        let mut total_removed = 0u64;

        {
            let mut metrics = self.metrics.lock().unwrap();
            let initial_len = metrics.len();
            metrics.retain(|m| m.timestamp > before);
            total_removed += (initial_len - metrics.len()) as u64;
        }

        {
            let mut aggregated = self.aggregated.lock().unwrap();
            let initial_len = aggregated.len();
            aggregated.retain(|m| m.timestamp > before);
            total_removed += (initial_len - aggregated.len()) as u64;
        }

        {
            let mut system_perf = self.system_performance.lock().unwrap();
            let initial_len = system_perf.len();
            system_perf.retain(|p| p.timestamp > before);
            total_removed += (initial_len - system_perf.len()) as u64;
        }

        Ok(total_removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_request_tracking() {
        let config = PerformanceConfig::default();
        let repository = Arc::new(MockPerformanceRepository::new());
        let monitor = PerformanceMonitor::new(config, repository);

        let request_id = "test_request_123".to_string();

        monitor
            .start_request(
                request_id.clone(),
                "/api/test".to_string(),
                "GET".to_string(),
                Some("user123".to_string()),
                Some("session456".to_string()),
            )
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        monitor
            .end_request(request_id, 200, 1024, 2048, None)
            .await
            .unwrap();

        monitor.flush_metrics().await.unwrap();

        // Check that metrics were recorded
        let metrics = monitor
            .repository
            .get_metrics(Some("/api/test"), Utc::now() - Duration::hours(1), None)
            .await
            .unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].endpoint, "/api/test");
        assert_eq!(metrics[0].status_code, 200);
        assert!(metrics[0].response_time_ms >= 50);
    }

    #[tokio::test]
    async fn test_system_performance_storage() {
        let config = PerformanceConfig::default();
        let repository = Arc::new(MockPerformanceRepository::new());
        let monitor = PerformanceMonitor::new(config, repository.clone());

        let performance = SystemPerformance {
            timestamp: Utc::now(),
            cpu_usage_percent: 25.0,
            memory_usage_mb: 1024,
            memory_usage_percent: 50.0,
            heap_size_mb: 512,
            garbage_collections: 5,
            thread_count: 10,
            active_connections: 100,
            connection_pool_usage: 75.0,
            database_connections: 5,
            cache_hit_ratio: 95.0,
            disk_io_read_mb: 100,
            disk_io_write_mb: 50,
            network_io_rx_mb: 200,
            network_io_tx_mb: 150,
        };

        monitor
            .record_system_performance(performance.clone())
            .await
            .unwrap();

        let stored = repository
            .get_system_performance(Utc::now() - Duration::hours(1))
            .await
            .unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].cpu_usage_percent, 25.0);
        assert_eq!(stored[0].memory_usage_mb, 1024);
    }

    #[tokio::test]
    async fn test_endpoint_performance_summary() {
        let config = PerformanceConfig::default();
        let repository = Arc::new(MockPerformanceRepository::new());
        let monitor = PerformanceMonitor::new(config, repository.clone());

        // Add some test metrics
        let metrics = vec![
            PerformanceMetrics {
                timestamp: Utc::now(),
                endpoint: "/api/test".to_string(),
                method: "GET".to_string(),
                response_time_ms: 100,
                status_code: 200,
                request_size_bytes: 1024,
                response_size_bytes: 2048,
                user_id: Some("user1".to_string()),
                session_id: Some("session1".to_string()),
                error_message: None,
            },
            PerformanceMetrics {
                timestamp: Utc::now(),
                endpoint: "/api/test".to_string(),
                method: "GET".to_string(),
                response_time_ms: 200,
                status_code: 500,
                request_size_bytes: 1024,
                response_size_bytes: 512,
                user_id: Some("user2".to_string()),
                session_id: Some("session2".to_string()),
                error_message: Some("Internal error".to_string()),
            },
        ];

        repository.store_metrics(&metrics).await.unwrap();

        let summary = monitor
            .get_endpoint_performance("/api/test", 1)
            .await
            .unwrap();

        assert_eq!(summary.endpoint, "/api/test");
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.avg_response_time_ms, 150.0);
        assert_eq!(summary.min_response_time_ms, 100);
        assert_eq!(summary.max_response_time_ms, 200);
        assert_eq!(summary.success_rate, 50.0);
        assert_eq!(summary.error_count, 1);
    }

    #[test]
    fn test_performance_config_defaults() {
        let config = PerformanceConfig::default();

        assert!(config.enabled);
        assert_eq!(config.sample_rate, 1.0);
        assert_eq!(config.buffer_size, 10000);
        assert_eq!(config.aggregation_window_minutes, 5);
        assert_eq!(config.retention_hours, 24);
    }

    #[test]
    fn test_alert_thresholds_defaults() {
        let thresholds = AlertThresholds::default();

        assert_eq!(thresholds.response_time_warning_ms, 1000);
        assert_eq!(thresholds.response_time_critical_ms, 3000);
        assert_eq!(thresholds.error_rate_warning_percent, 5.0);
        assert_eq!(thresholds.cpu_usage_critical_percent, 85.0);
    }

    #[test]
    fn test_percentile_calculation() {
        let config = PerformanceConfig::default();
        let repository = Arc::new(MockPerformanceRepository::new());
        let monitor = PerformanceMonitor::new(config, repository);

        let values = vec![100, 200, 300, 400, 500];

        assert_eq!(monitor.percentile(&values, 0.5), 300.0); // median
        assert_eq!(monitor.percentile(&values, 0.0), 100.0); // min
        assert_eq!(monitor.percentile(&values, 1.0), 500.0); // max

        // Empty array
        assert_eq!(monitor.percentile(&[], 0.5), 0.0);
    }

    #[tokio::test]
    async fn test_metrics_cleanup() {
        let config = PerformanceConfig::default();
        let repository = Arc::new(MockPerformanceRepository::new());
        let monitor = PerformanceMonitor::new(config, repository.clone());

        // Add old metrics
        let old_metrics = vec![PerformanceMetrics {
            timestamp: Utc::now() - Duration::days(2),
            endpoint: "/api/old".to_string(),
            method: "GET".to_string(),
            response_time_ms: 100,
            status_code: 200,
            request_size_bytes: 1024,
            response_size_bytes: 2048,
            user_id: None,
            session_id: None,
            error_message: None,
        }];

        repository.store_metrics(&old_metrics).await.unwrap();

        let removed = monitor.cleanup_old_metrics().await.unwrap();
        assert_eq!(removed, 1);

        let remaining = repository
            .get_metrics(None, Utc::now() - Duration::days(3), None)
            .await
            .unwrap();
        assert_eq!(remaining.len(), 0);
    }
}
