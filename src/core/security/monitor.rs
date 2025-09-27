use crate::utils::error::{ErpError, ErpResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sysinfo::{CpuExt, DiskExt, NetworkExt, NetworksExt, System, SystemExt};
use tokio::time::{sleep, Duration as TokioDuration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_available: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub disk_available: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub process_count: usize,
    pub uptime_seconds: u64,
    pub load_average: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub timestamp: DateTime<Utc>,
    pub active_connections: u32,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub memory_usage_mb: u64,
    pub heap_size_mb: u64,
    pub thread_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub timestamp: DateTime<Utc>,
    pub failed_login_attempts: u32,
    pub blocked_ips: u32,
    pub rate_limit_violations: u32,
    pub security_violations: u32,
    pub active_sessions: u32,
    pub suspicious_activities: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub service: String,
    pub status: HealthState,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, String>,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthState {
    Healthy,
    Warning,
    Critical,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub title: String,
    pub description: String,
    pub metrics: HashMap<String, String>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertCategory {
    System,
    Application,
    Security,
    Database,
    Network,
}

#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub collection_interval_seconds: u64,
    pub retention_hours: i64,
    pub cpu_warning_threshold: f32,
    pub cpu_critical_threshold: f32,
    pub memory_warning_threshold: f32,
    pub memory_critical_threshold: f32,
    pub disk_warning_threshold: f32,
    pub disk_critical_threshold: f32,
    pub response_time_warning_ms: u64,
    pub response_time_critical_ms: u64,
    pub error_rate_warning: f64,
    pub error_rate_critical: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_seconds: 30,
            retention_hours: 24,
            cpu_warning_threshold: 70.0,
            cpu_critical_threshold: 85.0,
            memory_warning_threshold: 80.0,
            memory_critical_threshold: 90.0,
            disk_warning_threshold: 80.0,
            disk_critical_threshold: 90.0,
            response_time_warning_ms: 1000,
            response_time_critical_ms: 3000,
            error_rate_warning: 5.0,
            error_rate_critical: 10.0,
        }
    }
}

#[async_trait::async_trait]
pub trait MetricsRepository: Send + Sync {
    async fn store_system_metrics(&self, metrics: &SystemMetrics) -> ErpResult<()>;
    async fn store_application_metrics(&self, metrics: &ApplicationMetrics) -> ErpResult<()>;
    async fn store_security_metrics(&self, metrics: &SecurityMetrics) -> ErpResult<()>;
    async fn store_health_status(&self, status: &HealthStatus) -> ErpResult<()>;
    async fn store_alert(&self, alert: &Alert) -> ErpResult<()>;

    async fn get_system_metrics(&self, since: DateTime<Utc>) -> ErpResult<Vec<SystemMetrics>>;
    async fn get_application_metrics(
        &self,
        since: DateTime<Utc>,
    ) -> ErpResult<Vec<ApplicationMetrics>>;
    async fn get_security_metrics(&self, since: DateTime<Utc>) -> ErpResult<Vec<SecurityMetrics>>;
    async fn get_health_status(&self, service: Option<&str>) -> ErpResult<Vec<HealthStatus>>;
    async fn get_alerts(&self, resolved: Option<bool>) -> ErpResult<Vec<Alert>>;

    async fn cleanup_old_metrics(&self, before: DateTime<Utc>) -> ErpResult<u64>;
}

pub struct MonitoringService {
    config: MonitoringConfig,
    repository: Box<dyn MetricsRepository>,
    system: Arc<Mutex<System>>,
    application_metrics: Arc<Mutex<ApplicationMetrics>>,
    security_metrics: Arc<Mutex<SecurityMetrics>>,
    health_checks: Arc<Mutex<Vec<Box<dyn HealthCheck>>>>,
    active_alerts: Arc<Mutex<HashMap<String, Alert>>>,
}

#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> ErpResult<HealthStatus>;
    fn name(&self) -> &str;
}

impl MonitoringService {
    pub fn new(config: MonitoringConfig, repository: Box<dyn MetricsRepository>) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            config,
            repository,
            system: Arc::new(Mutex::new(system)),
            application_metrics: Arc::new(Mutex::new(ApplicationMetrics::default())),
            security_metrics: Arc::new(Mutex::new(SecurityMetrics::default())),
            health_checks: Arc::new(Mutex::new(Vec::new())),
            active_alerts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_health_check(&self, check: Box<dyn HealthCheck>) {
        let mut checks = self.health_checks.lock().unwrap();
        checks.push(check);
        info!("Added health check: {}", checks.last().unwrap().name());
    }

    pub async fn start_monitoring(&self) -> ErpResult<()> {
        if !self.config.enabled {
            info!("Monitoring is disabled");
            return Ok(());
        }

        info!(
            "Starting monitoring service with interval: {} seconds",
            self.config.collection_interval_seconds
        );

        let interval = TokioDuration::from_secs(self.config.collection_interval_seconds);

        loop {
            if let Err(e) = self.collect_and_store_metrics().await {
                error!("Failed to collect metrics: {}", e);
            }

            if let Err(e) = self.run_health_checks().await {
                error!("Failed to run health checks: {}", e);
            }

            if let Err(e) = self.check_alerts().await {
                error!("Failed to check alerts: {}", e);
            }

            if let Err(e) = self.cleanup_old_data().await {
                warn!("Failed to cleanup old data: {}", e);
            }

            sleep(interval).await;
        }
    }

    async fn collect_and_store_metrics(&self) -> ErpResult<()> {
        // Collect system metrics
        let system_metrics = self.collect_system_metrics()?;
        self.repository
            .store_system_metrics(&system_metrics)
            .await?;

        // Store application metrics
        let app_metrics = {
            let metrics = self.application_metrics.lock().unwrap();
            metrics.clone()
        };
        self.repository
            .store_application_metrics(&app_metrics)
            .await?;

        // Store security metrics
        let sec_metrics = {
            let metrics = self.security_metrics.lock().unwrap();
            metrics.clone()
        };
        self.repository.store_security_metrics(&sec_metrics).await?;

        debug!("Metrics collected and stored successfully");
        Ok(())
    }

    fn collect_system_metrics(&self) -> ErpResult<SystemMetrics> {
        let mut system = self
            .system
            .lock()
            .map_err(|_| ErpError::internal("Lock poisoned"))?;
        system.refresh_all();

        let cpu_usage = system.global_cpu_info().cpu_usage();

        let memory_total = system.total_memory();
        let memory_used = system.used_memory();
        let memory_available = system.available_memory();

        let disk_total = system.disks().iter().map(|d| d.total_space()).sum();
        let disk_available = system.disks().iter().map(|d| d.available_space()).sum();
        let disk_used = disk_total - disk_available;

        let (network_rx_bytes, network_tx_bytes) =
            system.networks().iter().fold((0, 0), |(rx, tx), (_, net)| {
                (rx + net.received(), tx + net.transmitted())
            });

        let process_count = system.processes().len();
        let uptime_seconds = system.uptime();

        // Simple load average calculation (not available on all platforms)
        let load_average = cpu_usage as f64 / 100.0;

        Ok(SystemMetrics {
            timestamp: Utc::now(),
            cpu_usage,
            memory_total,
            memory_used,
            memory_available,
            disk_total,
            disk_used,
            disk_available,
            network_rx_bytes,
            network_tx_bytes,
            process_count,
            uptime_seconds,
            load_average,
        })
    }

    async fn run_health_checks(&self) -> ErpResult<()> {
        let checks = {
            let health_checks = self.health_checks.lock().unwrap();
            health_checks
                .iter()
                .map(|c| c.name().to_string())
                .collect::<Vec<_>>()
        };

        for check_name in checks {
            let health_checks = self.health_checks.lock().unwrap();
            if let Some(check) = health_checks.iter().find(|c| c.name() == check_name) {
                match check.check().await {
                    Ok(status) => {
                        self.repository.store_health_status(&status).await?;
                        debug!("Health check {} completed: {:?}", check_name, status.status);
                    }
                    Err(e) => {
                        warn!("Health check {} failed: {}", check_name, e);
                        let status = HealthStatus {
                            service: check_name.clone(),
                            status: HealthState::Critical,
                            timestamp: Utc::now(),
                            details: [("error".to_string(), e.to_string())]
                                .iter()
                                .cloned()
                                .collect(),
                            response_time_ms: None,
                        };
                        self.repository.store_health_status(&status).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn check_alerts(&self) -> ErpResult<()> {
        let system_metrics = self.collect_system_metrics()?;

        // Check CPU alerts
        if system_metrics.cpu_usage > self.config.cpu_critical_threshold {
            self.create_alert(
                AlertSeverity::Critical,
                AlertCategory::System,
                "High CPU Usage".to_string(),
                format!(
                    "CPU usage is {}%, which exceeds critical threshold of {}%",
                    system_metrics.cpu_usage, self.config.cpu_critical_threshold
                ),
                [(
                    "cpu_usage".to_string(),
                    system_metrics.cpu_usage.to_string(),
                )]
                .iter()
                .cloned()
                .collect(),
            )
            .await?;
        } else if system_metrics.cpu_usage > self.config.cpu_warning_threshold {
            self.create_alert(
                AlertSeverity::Warning,
                AlertCategory::System,
                "Elevated CPU Usage".to_string(),
                format!(
                    "CPU usage is {}%, which exceeds warning threshold of {}%",
                    system_metrics.cpu_usage, self.config.cpu_warning_threshold
                ),
                [(
                    "cpu_usage".to_string(),
                    system_metrics.cpu_usage.to_string(),
                )]
                .iter()
                .cloned()
                .collect(),
            )
            .await?;
        }

        // Check memory alerts
        let memory_usage_percent =
            (system_metrics.memory_used as f32 / system_metrics.memory_total as f32) * 100.0;
        if memory_usage_percent > self.config.memory_critical_threshold {
            self.create_alert(
                AlertSeverity::Critical,
                AlertCategory::System,
                "High Memory Usage".to_string(),
                format!(
                    "Memory usage is {:.1}%, which exceeds critical threshold of {}%",
                    memory_usage_percent, self.config.memory_critical_threshold
                ),
                [(
                    "memory_usage_percent".to_string(),
                    memory_usage_percent.to_string(),
                )]
                .iter()
                .cloned()
                .collect(),
            )
            .await?;
        }

        // Check disk alerts
        let disk_usage_percent =
            (system_metrics.disk_used as f32 / system_metrics.disk_total as f32) * 100.0;
        if disk_usage_percent > self.config.disk_critical_threshold {
            self.create_alert(
                AlertSeverity::Critical,
                AlertCategory::System,
                "High Disk Usage".to_string(),
                format!(
                    "Disk usage is {:.1}%, which exceeds critical threshold of {}%",
                    disk_usage_percent, self.config.disk_critical_threshold
                ),
                [(
                    "disk_usage_percent".to_string(),
                    disk_usage_percent.to_string(),
                )]
                .iter()
                .cloned()
                .collect(),
            )
            .await?;
        }

        Ok(())
    }

    async fn create_alert(
        &self,
        severity: AlertSeverity,
        category: AlertCategory,
        title: String,
        description: String,
        metrics: HashMap<String, String>,
    ) -> ErpResult<()> {
        let alert_key = format!("{:?}-{}", category, title);

        let mut active_alerts = self.active_alerts.lock().unwrap();

        // Check if alert already exists
        if active_alerts.contains_key(&alert_key) {
            return Ok(());
        }

        let alert = Alert {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity,
            category,
            title,
            description,
            metrics,
            resolved: false,
            resolved_at: None,
        };

        active_alerts.insert(alert_key, alert.clone());
        drop(active_alerts);

        self.repository.store_alert(&alert).await?;

        match alert.severity {
            AlertSeverity::Critical => error!("ALERT: {}", alert.description),
            AlertSeverity::Error => error!("ALERT: {}", alert.description),
            AlertSeverity::Warning => warn!("ALERT: {}", alert.description),
            AlertSeverity::Info => info!("ALERT: {}", alert.description),
        }

        Ok(())
    }

    pub async fn resolve_alert(&self, alert_id: Uuid) -> ErpResult<()> {
        let mut active_alerts = self.active_alerts.lock().unwrap();

        // Find and remove the alert
        let mut found_alert = None;
        active_alerts.retain(|_, alert| {
            if alert.id == alert_id {
                found_alert = Some(alert.clone());
                false
            } else {
                true
            }
        });

        if let Some(mut alert) = found_alert {
            alert.resolved = true;
            alert.resolved_at = Some(Utc::now());

            self.repository.store_alert(&alert).await?;
            info!("Alert resolved: {}", alert.title);
        }

        Ok(())
    }

    pub async fn get_active_alerts(&self) -> ErpResult<Vec<Alert>> {
        let active_alerts = self.active_alerts.lock().unwrap();
        Ok(active_alerts.values().cloned().collect())
    }

    pub fn update_application_metrics<F>(&self, update_fn: F) -> ErpResult<()>
    where
        F: FnOnce(&mut ApplicationMetrics),
    {
        let mut metrics = self
            .application_metrics
            .lock()
            .map_err(|_| ErpError::internal("Lock poisoned"))?;
        metrics.timestamp = Utc::now();
        update_fn(&mut metrics);
        Ok(())
    }

    pub fn update_security_metrics<F>(&self, update_fn: F) -> ErpResult<()>
    where
        F: FnOnce(&mut SecurityMetrics),
    {
        let mut metrics = self
            .security_metrics
            .lock()
            .map_err(|_| ErpError::internal("Lock poisoned"))?;
        metrics.timestamp = Utc::now();
        update_fn(&mut metrics);
        Ok(())
    }

    async fn cleanup_old_data(&self) -> ErpResult<()> {
        let cutoff = Utc::now() - Duration::hours(self.config.retention_hours);
        let removed = self.repository.cleanup_old_metrics(cutoff).await?;
        if removed > 0 {
            debug!("Cleaned up {} old metric records", removed);
        }
        Ok(())
    }

    pub async fn get_system_health_summary(&self) -> ErpResult<SystemHealthSummary> {
        let system_metrics = self.collect_system_metrics()?;
        let health_statuses = self.repository.get_health_status(None).await?;
        let active_alerts = self.get_active_alerts().await?;

        let critical_alerts = active_alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Critical)
            .count();

        let warning_alerts = active_alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Warning)
            .count();

        let healthy_services = health_statuses
            .iter()
            .filter(|s| s.status == HealthState::Healthy)
            .count();

        let total_services = health_statuses.len();

        Ok(SystemHealthSummary {
            timestamp: Utc::now(),
            overall_status: if critical_alerts > 0 {
                HealthState::Critical
            } else if warning_alerts > 0 {
                HealthState::Warning
            } else {
                HealthState::Healthy
            },
            cpu_usage: system_metrics.cpu_usage,
            memory_usage_percent: (system_metrics.memory_used as f32
                / system_metrics.memory_total as f32)
                * 100.0,
            disk_usage_percent: (system_metrics.disk_used as f32
                / system_metrics.disk_total as f32)
                * 100.0,
            healthy_services,
            total_services,
            active_alerts: active_alerts.len(),
            critical_alerts,
            warning_alerts,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthSummary {
    pub timestamp: DateTime<Utc>,
    pub overall_status: HealthState,
    pub cpu_usage: f32,
    pub memory_usage_percent: f32,
    pub disk_usage_percent: f32,
    pub healthy_services: usize,
    pub total_services: usize,
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
}

impl Default for ApplicationMetrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            active_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            error_rate: 0.0,
            memory_usage_mb: 0,
            heap_size_mb: 0,
            thread_count: 1,
        }
    }
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            failed_login_attempts: 0,
            blocked_ips: 0,
            rate_limit_violations: 0,
            security_violations: 0,
            active_sessions: 0,
            suspicious_activities: 0,
        }
    }
}

// Example health checks
pub struct DatabaseHealthCheck {
    name: String,
}

impl DatabaseHealthCheck {
    pub fn new() -> Self {
        Self {
            name: "database".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> ErpResult<HealthStatus> {
        let start_time = std::time::Instant::now();

        // Simulate database health check
        tokio::time::sleep(TokioDuration::from_millis(10)).await;

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        // Simulate random health status
        let status = if response_time_ms > 100 {
            HealthState::Warning
        } else {
            HealthState::Healthy
        };

        Ok(HealthStatus {
            service: self.name.clone(),
            status,
            timestamp: Utc::now(),
            details: [("response_time_ms".to_string(), response_time_ms.to_string())]
                .iter()
                .cloned()
                .collect(),
            response_time_ms: Some(response_time_ms),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// Mock repository for testing
#[derive(Debug, Clone)]
pub struct MockMetricsRepository {
    system_metrics: Arc<Mutex<Vec<SystemMetrics>>>,
    application_metrics: Arc<Mutex<Vec<ApplicationMetrics>>>,
    security_metrics: Arc<Mutex<Vec<SecurityMetrics>>>,
    health_statuses: Arc<Mutex<Vec<HealthStatus>>>,
    alerts: Arc<Mutex<Vec<Alert>>>,
}

impl MockMetricsRepository {
    pub fn new() -> Self {
        Self {
            system_metrics: Arc::new(Mutex::new(Vec::new())),
            application_metrics: Arc::new(Mutex::new(Vec::new())),
            security_metrics: Arc::new(Mutex::new(Vec::new())),
            health_statuses: Arc::new(Mutex::new(Vec::new())),
            alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl MetricsRepository for MockMetricsRepository {
    async fn store_system_metrics(&self, metrics: &SystemMetrics) -> ErpResult<()> {
        let mut stored_metrics = self.system_metrics.lock().unwrap();
        stored_metrics.push(metrics.clone());
        Ok(())
    }

    async fn store_application_metrics(&self, metrics: &ApplicationMetrics) -> ErpResult<()> {
        let mut stored_metrics = self.application_metrics.lock().unwrap();
        stored_metrics.push(metrics.clone());
        Ok(())
    }

    async fn store_security_metrics(&self, metrics: &SecurityMetrics) -> ErpResult<()> {
        let mut stored_metrics = self.security_metrics.lock().unwrap();
        stored_metrics.push(metrics.clone());
        Ok(())
    }

    async fn store_health_status(&self, status: &HealthStatus) -> ErpResult<()> {
        let mut statuses = self.health_statuses.lock().unwrap();
        statuses.push(status.clone());
        Ok(())
    }

    async fn store_alert(&self, alert: &Alert) -> ErpResult<()> {
        let mut alerts = self.alerts.lock().unwrap();
        alerts.push(alert.clone());
        Ok(())
    }

    async fn get_system_metrics(&self, since: DateTime<Utc>) -> ErpResult<Vec<SystemMetrics>> {
        let metrics = self.system_metrics.lock().unwrap();
        Ok(metrics
            .iter()
            .filter(|m| m.timestamp > since)
            .cloned()
            .collect())
    }

    async fn get_application_metrics(
        &self,
        since: DateTime<Utc>,
    ) -> ErpResult<Vec<ApplicationMetrics>> {
        let metrics = self.application_metrics.lock().unwrap();
        Ok(metrics
            .iter()
            .filter(|m| m.timestamp > since)
            .cloned()
            .collect())
    }

    async fn get_security_metrics(&self, since: DateTime<Utc>) -> ErpResult<Vec<SecurityMetrics>> {
        let metrics = self.security_metrics.lock().unwrap();
        Ok(metrics
            .iter()
            .filter(|m| m.timestamp > since)
            .cloned()
            .collect())
    }

    async fn get_health_status(&self, service: Option<&str>) -> ErpResult<Vec<HealthStatus>> {
        let statuses = self.health_statuses.lock().unwrap();
        if let Some(service_name) = service {
            Ok(statuses
                .iter()
                .filter(|s| s.service == service_name)
                .cloned()
                .collect())
        } else {
            Ok(statuses.clone())
        }
    }

    async fn get_alerts(&self, resolved: Option<bool>) -> ErpResult<Vec<Alert>> {
        let alerts = self.alerts.lock().unwrap();
        if let Some(resolved_filter) = resolved {
            Ok(alerts
                .iter()
                .filter(|a| a.resolved == resolved_filter)
                .cloned()
                .collect())
        } else {
            Ok(alerts.clone())
        }
    }

    async fn cleanup_old_metrics(&self, before: DateTime<Utc>) -> ErpResult<u64> {
        let mut total_removed = 0u64;

        {
            let mut metrics = self.system_metrics.lock().unwrap();
            let initial_len = metrics.len();
            metrics.retain(|m| m.timestamp > before);
            total_removed += (initial_len - metrics.len()) as u64;
        }

        {
            let mut metrics = self.application_metrics.lock().unwrap();
            let initial_len = metrics.len();
            metrics.retain(|m| m.timestamp > before);
            total_removed += (initial_len - metrics.len()) as u64;
        }

        {
            let mut metrics = self.security_metrics.lock().unwrap();
            let initial_len = metrics.len();
            metrics.retain(|m| m.timestamp > before);
            total_removed += (initial_len - metrics.len()) as u64;
        }

        Ok(total_removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_creation() {
        let metrics = SystemMetrics {
            timestamp: Utc::now(),
            cpu_usage: 50.0,
            memory_total: 8_000_000_000,
            memory_used: 4_000_000_000,
            memory_available: 4_000_000_000,
            disk_total: 500_000_000_000,
            disk_used: 250_000_000_000,
            disk_available: 250_000_000_000,
            network_rx_bytes: 1_000_000,
            network_tx_bytes: 500_000,
            process_count: 150,
            uptime_seconds: 86400,
            load_average: 0.5,
        };

        assert_eq!(metrics.cpu_usage, 50.0);
        assert_eq!(metrics.process_count, 150);
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity: AlertSeverity::Critical,
            category: AlertCategory::System,
            title: "High CPU Usage".to_string(),
            description: "CPU usage exceeds 90%".to_string(),
            metrics: [("cpu_usage".to_string(), "95.0".to_string())]
                .iter()
                .cloned()
                .collect(),
            resolved: false,
            resolved_at: None,
        };

        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.category, AlertCategory::System);
        assert!(!alert.resolved);
        assert!(alert.resolved_at.is_none());
    }

    #[test]
    fn test_health_status_creation() {
        let status = HealthStatus {
            service: "database".to_string(),
            status: HealthState::Healthy,
            timestamp: Utc::now(),
            details: [("connection_pool_size".to_string(), "10".to_string())]
                .iter()
                .cloned()
                .collect(),
            response_time_ms: Some(25),
        };

        assert_eq!(status.status, HealthState::Healthy);
        assert_eq!(status.service, "database");
        assert_eq!(status.response_time_ms, Some(25));
    }

    #[tokio::test]
    async fn test_mock_repository() {
        let repository = MockMetricsRepository::new();

        let metrics = SystemMetrics {
            timestamp: Utc::now(),
            cpu_usage: 25.0,
            memory_total: 1000,
            memory_used: 500,
            memory_available: 500,
            disk_total: 1000,
            disk_used: 300,
            disk_available: 700,
            network_rx_bytes: 1000,
            network_tx_bytes: 500,
            process_count: 10,
            uptime_seconds: 3600,
            load_average: 0.25,
        };

        repository.store_system_metrics(&metrics).await.unwrap();

        let since = Utc::now() - Duration::minutes(1);
        let retrieved = repository.get_system_metrics(since).await.unwrap();

        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].cpu_usage, 25.0);
    }

    #[tokio::test]
    async fn test_database_health_check() {
        let health_check = DatabaseHealthCheck::new();
        let status = health_check.check().await.unwrap();

        assert_eq!(status.service, "database");
        assert!(matches!(
            status.status,
            HealthState::Healthy | HealthState::Warning
        ));
        assert!(status.response_time_ms.is_some());
    }

    #[test]
    fn test_monitoring_config() {
        let config = MonitoringConfig::default();

        assert!(config.enabled);
        assert_eq!(config.collection_interval_seconds, 30);
        assert_eq!(config.cpu_warning_threshold, 70.0);
        assert_eq!(config.memory_critical_threshold, 90.0);
    }

    #[tokio::test]
    async fn test_monitoring_service_creation() {
        let config = MonitoringConfig::default();
        let repository = Box::new(MockMetricsRepository::new());
        let service = MonitoringService::new(config, repository);

        let health_check = Box::new(DatabaseHealthCheck::new());
        service.add_health_check(health_check);

        // Test metrics update
        let result = service.update_application_metrics(|metrics| {
            metrics.active_connections = 10;
            metrics.total_requests = 100;
        });

        assert!(result.is_ok());
    }
}
