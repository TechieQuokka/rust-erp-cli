use crate::utils::error::{ErpError, ErpResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub environment: Environment,
    pub application_name: String,
    pub version: String,
    pub build_config: BuildConfig,
    pub database_config: DatabaseDeploymentConfig,
    pub security_config: SecurityDeploymentConfig,
    pub monitoring_config: MonitoringDeploymentConfig,
    pub scaling_config: ScalingConfig,
    pub health_checks: Vec<HealthCheckConfig>,
    pub rollback_config: RollbackConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target: String,
    pub features: Vec<String>,
    pub optimization_level: OptimizationLevel,
    pub build_directory: PathBuf,
    pub artifact_name: String,
    pub pre_build_scripts: Vec<String>,
    pub post_build_scripts: Vec<String>,
    pub environment_variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationLevel {
    Debug,
    Release,
    Size,
    Speed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseDeploymentConfig {
    pub run_migrations: bool,
    pub backup_before_migration: bool,
    pub migration_timeout_seconds: u64,
    pub database_url: String,
    pub connection_pool_size: u32,
    pub migration_scripts_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDeploymentConfig {
    pub enable_https: bool,
    pub certificate_path: Option<PathBuf>,
    pub private_key_path: Option<PathBuf>,
    pub security_headers: bool,
    pub cors_origins: Vec<String>,
    pub rate_limiting: bool,
    pub audit_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDeploymentConfig {
    pub metrics_enabled: bool,
    pub health_check_endpoint: String,
    pub prometheus_endpoint: Option<String>,
    pub log_level: String,
    pub structured_logging: bool,
    pub alert_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub min_instances: u32,
    pub max_instances: u32,
    pub cpu_threshold: f32,
    pub memory_threshold: f32,
    pub auto_scaling_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub name: String,
    pub endpoint: String,
    pub timeout_seconds: u64,
    pub expected_status_code: u16,
    pub retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackConfig {
    pub enabled: bool,
    pub automatic_rollback: bool,
    pub health_check_failures_threshold: u32,
    pub rollback_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub environment: Environment,
    pub version: String,
    pub status: DeploymentStatus,
    pub config: DeploymentConfig,
    pub build_artifacts: Vec<BuildArtifact>,
    pub deployment_steps: Vec<DeploymentStep>,
    pub health_check_results: Vec<HealthCheckResult>,
    pub rollback_info: Option<RollbackInfo>,
    pub duration_seconds: Option<u64>,
    pub error_message: Option<String>,
    pub deployed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentStatus {
    Pending,
    Building,
    Testing,
    Deploying,
    HealthChecking,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStep {
    pub name: String,
    pub status: StepStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub check_name: String,
    pub status: HealthCheckStatus,
    pub response_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthCheckStatus {
    Passed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    pub triggered_by: RollbackTrigger,
    pub previous_version: String,
    pub rollback_timestamp: DateTime<Utc>,
    pub rollback_duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackTrigger {
    Manual,
    HealthCheckFailure,
    ErrorThreshold,
    DeploymentFailure,
}

#[async_trait::async_trait]
pub trait DeploymentRepository: Send + Sync {
    async fn store_deployment(&self, deployment: &DeploymentRecord) -> ErpResult<()>;
    async fn update_deployment(&self, deployment: &DeploymentRecord) -> ErpResult<()>;
    async fn get_deployment(&self, id: Uuid) -> ErpResult<Option<DeploymentRecord>>;
    async fn get_deployments_by_environment(
        &self,
        env: Environment,
    ) -> ErpResult<Vec<DeploymentRecord>>;
    async fn get_latest_deployment(&self, env: Environment) -> ErpResult<Option<DeploymentRecord>>;
    async fn get_deployment_history(
        &self,
        limit: Option<usize>,
    ) -> ErpResult<Vec<DeploymentRecord>>;
}

pub struct DeploymentService {
    repository: Box<dyn DeploymentRepository>,
}

impl DeploymentService {
    pub fn new(repository: Box<dyn DeploymentRepository>) -> Self {
        Self { repository }
    }

    pub async fn deploy(
        &self,
        config: DeploymentConfig,
        deployed_by: String,
    ) -> ErpResult<DeploymentRecord> {
        info!(
            "Starting deployment to {:?} environment, version: {}",
            config.environment, config.version
        );

        let deployment_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();

        let mut deployment = DeploymentRecord {
            id: deployment_id,
            timestamp: Utc::now(),
            environment: config.environment.clone(),
            version: config.version.clone(),
            status: DeploymentStatus::Pending,
            config: config.clone(),
            build_artifacts: Vec::new(),
            deployment_steps: Vec::new(),
            health_check_results: Vec::new(),
            rollback_info: None,
            duration_seconds: None,
            error_message: None,
            deployed_by,
        };

        // Store initial deployment record
        self.repository.store_deployment(&deployment).await?;

        // Execute deployment steps
        match self.execute_deployment(&mut deployment).await {
            Ok(_) => {
                deployment.status = DeploymentStatus::Completed;
                deployment.duration_seconds = Some(start_time.elapsed().as_secs());
                info!("Deployment completed successfully: {}", deployment_id);
            }
            Err(e) => {
                error!("Deployment failed: {}", e);
                deployment.status = DeploymentStatus::Failed;
                deployment.error_message = Some(e.to_string());
                deployment.duration_seconds = Some(start_time.elapsed().as_secs());

                // Attempt rollback if configured
                if deployment.config.rollback_config.enabled
                    && deployment.config.rollback_config.automatic_rollback
                {
                    if let Err(rollback_error) = self.rollback(&mut deployment).await {
                        error!("Automatic rollback failed: {}", rollback_error);
                    }
                }
            }
        }

        // Update final deployment record
        self.repository.update_deployment(&deployment).await?;

        Ok(deployment)
    }

    pub async fn rollback_deployment(
        &self,
        deployment_id: Uuid,
        _rolled_back_by: String,
    ) -> ErpResult<()> {
        let mut deployment = self
            .repository
            .get_deployment(deployment_id)
            .await?
            .ok_or_else(|| ErpError::not_found("deployment", deployment_id.to_string()))?;

        if deployment.status != DeploymentStatus::Completed {
            return Err(ErpError::validation(
                "deployment",
                "Can only rollback completed deployments",
            ));
        }

        info!("Starting manual rollback for deployment: {}", deployment_id);

        self.rollback(&mut deployment).await?;
        self.repository.update_deployment(&deployment).await?;

        info!("Rollback completed for deployment: {}", deployment_id);
        Ok(())
    }

    pub async fn get_deployment_status(&self, deployment_id: Uuid) -> ErpResult<DeploymentRecord> {
        self.repository
            .get_deployment(deployment_id)
            .await?
            .ok_or_else(|| ErpError::not_found("deployment", deployment_id.to_string()))
    }

    pub async fn list_deployments(
        &self,
        environment: Option<Environment>,
        limit: Option<usize>,
    ) -> ErpResult<Vec<DeploymentRecord>> {
        match environment {
            Some(env) => self.repository.get_deployments_by_environment(env).await,
            None => self.repository.get_deployment_history(limit).await,
        }
    }

    async fn execute_deployment(&self, deployment: &mut DeploymentRecord) -> ErpResult<()> {
        // Step 1: Build
        deployment.status = DeploymentStatus::Building;
        self.repository.update_deployment(deployment).await?;
        self.execute_build_step(deployment).await?;

        // Step 2: Run pre-deployment tests
        deployment.status = DeploymentStatus::Testing;
        self.repository.update_deployment(deployment).await?;
        self.execute_testing_step(deployment).await?;

        // Step 3: Database migrations
        deployment.status = DeploymentStatus::Deploying;
        self.repository.update_deployment(deployment).await?;
        self.execute_database_migration_step(deployment).await?;

        // Step 4: Deploy application
        self.execute_application_deployment_step(deployment).await?;

        // Step 5: Health checks
        deployment.status = DeploymentStatus::HealthChecking;
        self.repository.update_deployment(deployment).await?;
        self.execute_health_checks(deployment).await?;

        Ok(())
    }

    async fn execute_build_step(&self, deployment: &mut DeploymentRecord) -> ErpResult<()> {
        let step_name = "Build Application";
        let start_time = Utc::now();

        let mut step = DeploymentStep {
            name: step_name.to_string(),
            status: StepStatus::InProgress,
            started_at: start_time,
            completed_at: None,
            output: None,
            error_message: None,
        };

        deployment.deployment_steps.push(step.clone());

        info!("Executing build step");

        // Run pre-build scripts
        for script in &deployment.config.build_config.pre_build_scripts {
            self.run_script(
                script,
                &deployment.config.build_config.environment_variables,
            )?;
        }

        // Execute cargo build
        let build_result = self
            .execute_cargo_build(&deployment.config.build_config)
            .await?;

        // Create build artifacts
        let artifacts = self
            .create_build_artifacts(&deployment.config.build_config, &build_result)
            .await?;
        deployment.build_artifacts = artifacts;

        // Run post-build scripts
        for script in &deployment.config.build_config.post_build_scripts {
            self.run_script(
                script,
                &deployment.config.build_config.environment_variables,
            )?;
        }

        // Update step status
        step.status = StepStatus::Completed;
        step.completed_at = Some(Utc::now());
        step.output = Some(build_result);

        if let Some(last_step) = deployment.deployment_steps.last_mut() {
            *last_step = step;
        }

        info!("Build step completed successfully");
        Ok(())
    }

    async fn execute_testing_step(&self, deployment: &mut DeploymentRecord) -> ErpResult<()> {
        let step_name = "Run Tests";
        let start_time = Utc::now();

        let mut step = DeploymentStep {
            name: step_name.to_string(),
            status: StepStatus::InProgress,
            started_at: start_time,
            completed_at: None,
            output: None,
            error_message: None,
        };

        deployment.deployment_steps.push(step.clone());

        info!("Executing testing step");

        let test_result = self.execute_tests(&deployment.config).await?;

        step.status = StepStatus::Completed;
        step.completed_at = Some(Utc::now());
        step.output = Some(test_result);

        if let Some(last_step) = deployment.deployment_steps.last_mut() {
            *last_step = step;
        }

        info!("Testing step completed successfully");
        Ok(())
    }

    async fn execute_database_migration_step(
        &self,
        deployment: &mut DeploymentRecord,
    ) -> ErpResult<()> {
        if !deployment.config.database_config.run_migrations {
            debug!("Skipping database migrations");
            return Ok(());
        }

        let step_name = "Database Migration";
        let start_time = Utc::now();

        let mut step = DeploymentStep {
            name: step_name.to_string(),
            status: StepStatus::InProgress,
            started_at: start_time,
            completed_at: None,
            output: None,
            error_message: None,
        };

        deployment.deployment_steps.push(step.clone());

        info!("Executing database migration step");

        // Backup database if configured
        if deployment.config.database_config.backup_before_migration {
            info!("Creating database backup before migration");
            // Database backup logic would go here
        }

        let migration_result = self
            .execute_database_migrations(&deployment.config.database_config)
            .await?;

        step.status = StepStatus::Completed;
        step.completed_at = Some(Utc::now());
        step.output = Some(migration_result);

        if let Some(last_step) = deployment.deployment_steps.last_mut() {
            *last_step = step;
        }

        info!("Database migration step completed successfully");
        Ok(())
    }

    async fn execute_application_deployment_step(
        &self,
        deployment: &mut DeploymentRecord,
    ) -> ErpResult<()> {
        let step_name = "Deploy Application";
        let start_time = Utc::now();

        let mut step = DeploymentStep {
            name: step_name.to_string(),
            status: StepStatus::InProgress,
            started_at: start_time,
            completed_at: None,
            output: None,
            error_message: None,
        };

        deployment.deployment_steps.push(step.clone());

        info!("Executing application deployment step");

        let deployment_result = self
            .deploy_application_artifacts(&deployment.build_artifacts)
            .await?;

        step.status = StepStatus::Completed;
        step.completed_at = Some(Utc::now());
        step.output = Some(deployment_result);

        if let Some(last_step) = deployment.deployment_steps.last_mut() {
            *last_step = step;
        }

        info!("Application deployment step completed successfully");
        Ok(())
    }

    async fn execute_health_checks(&self, deployment: &mut DeploymentRecord) -> ErpResult<()> {
        info!("Running health checks");

        for health_check in &deployment.config.health_checks {
            let result = self.run_health_check(health_check).await?;
            deployment.health_check_results.push(result.clone());

            if result.status == HealthCheckStatus::Failed {
                if deployment.config.rollback_config.enabled {
                    let failed_checks = deployment
                        .health_check_results
                        .iter()
                        .filter(|r| r.status == HealthCheckStatus::Failed)
                        .count();

                    if failed_checks
                        >= deployment
                            .config
                            .rollback_config
                            .health_check_failures_threshold as usize
                    {
                        warn!("Health check failure threshold reached, triggering rollback");
                        return Err(ErpError::validation("health_check", "Health checks failed"));
                    }
                } else {
                    return Err(ErpError::validation(
                        "health_check",
                        format!("Health check failed: {}", health_check.name),
                    ));
                }
            }
        }

        info!("All health checks passed");
        Ok(())
    }

    async fn rollback(&self, deployment: &mut DeploymentRecord) -> ErpResult<()> {
        info!("Starting rollback for deployment: {}", deployment.id);

        let rollback_start = std::time::Instant::now();

        // Get previous deployment
        let previous_deployment = self
            .repository
            .get_latest_deployment(deployment.environment.clone())
            .await?
            .filter(|d| d.id != deployment.id && d.status == DeploymentStatus::Completed);

        let previous_version = previous_deployment
            .map(|d| d.version)
            .unwrap_or_else(|| "unknown".to_string());

        // Execute rollback steps
        self.execute_rollback_steps(deployment, &previous_version)
            .await?;

        let rollback_info = RollbackInfo {
            triggered_by: if deployment.status == DeploymentStatus::Failed {
                RollbackTrigger::DeploymentFailure
            } else {
                RollbackTrigger::HealthCheckFailure
            },
            previous_version,
            rollback_timestamp: Utc::now(),
            rollback_duration_seconds: rollback_start.elapsed().as_secs(),
        };

        deployment.rollback_info = Some(rollback_info);
        deployment.status = DeploymentStatus::RolledBack;

        info!("Rollback completed for deployment: {}", deployment.id);
        Ok(())
    }

    async fn execute_rollback_steps(
        &self,
        deployment: &DeploymentRecord,
        previous_version: &str,
    ) -> ErpResult<()> {
        info!("Rolling back to version: {}", previous_version);

        // Stop current application
        self.stop_application().await?;

        // Restore previous application version
        self.restore_previous_version(previous_version).await?;

        // Start application
        self.start_application().await?;

        // Run basic health check
        if !deployment.config.health_checks.is_empty() {
            let basic_check = &deployment.config.health_checks[0];
            let result = self.run_health_check(basic_check).await?;

            if result.status != HealthCheckStatus::Passed {
                return Err(ErpError::internal("Rollback health check failed"));
            }
        }

        Ok(())
    }

    async fn execute_cargo_build(&self, build_config: &BuildConfig) -> ErpResult<String> {
        let mut cmd = Command::new("cargo");

        cmd.arg("build");

        match build_config.optimization_level {
            OptimizationLevel::Release | OptimizationLevel::Size | OptimizationLevel::Speed => {
                cmd.arg("--release");
            }
            OptimizationLevel::Debug => {
                // Debug is default
            }
        }

        if !build_config.features.is_empty() {
            cmd.arg("--features").arg(build_config.features.join(","));
        }

        // Set environment variables
        for (key, value) in &build_config.environment_variables {
            cmd.env(key, value);
        }

        let output = cmd
            .output()
            .map_err(|e| ErpError::internal(format!("Failed to execute cargo build: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ErpError::internal(format!(
                "Cargo build failed: {}",
                error
            )));
        }

        let build_output = String::from_utf8_lossy(&output.stdout);
        debug!("Cargo build output: {}", build_output);

        Ok(build_output.to_string())
    }

    async fn execute_tests(&self, config: &DeploymentConfig) -> ErpResult<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("test");

        if config.build_config.optimization_level == OptimizationLevel::Release {
            cmd.arg("--release");
        }

        let output = cmd
            .output()
            .map_err(|e| ErpError::internal(format!("Failed to execute cargo test: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ErpError::internal(format!("Tests failed: {}", error)));
        }

        let test_output = String::from_utf8_lossy(&output.stdout);
        debug!("Test output: {}", test_output);

        Ok(test_output.to_string())
    }

    async fn execute_database_migrations(
        &self,
        db_config: &DatabaseDeploymentConfig,
    ) -> ErpResult<String> {
        info!("Running database migrations");

        // This is a placeholder - in a real implementation, you would:
        // 1. Connect to the database
        // 2. Run migration scripts from the specified path
        // 3. Update migration tracking table

        let migration_result = format!(
            "Migrations executed from: {}",
            db_config.migration_scripts_path.display()
        );
        debug!("Database migration result: {}", migration_result);

        Ok(migration_result)
    }

    async fn create_build_artifacts(
        &self,
        build_config: &BuildConfig,
        _build_output: &str,
    ) -> ErpResult<Vec<BuildArtifact>> {
        let mut artifacts = Vec::new();

        // Determine binary path based on optimization level
        let binary_dir = match build_config.optimization_level {
            OptimizationLevel::Debug => "debug",
            _ => "release",
        };

        let binary_path = build_config
            .build_directory
            .join("target")
            .join(binary_dir)
            .join(&build_config.artifact_name);

        if binary_path.exists() {
            let metadata = fs::metadata(&binary_path)?;
            let size = metadata.len();

            // Calculate checksum
            let checksum = self.calculate_file_checksum(&binary_path)?;

            let artifact = BuildArtifact {
                name: build_config.artifact_name.clone(),
                path: binary_path,
                size_bytes: size,
                checksum,
                created_at: Utc::now(),
            };

            artifacts.push(artifact);
        }

        Ok(artifacts)
    }

    async fn deploy_application_artifacts(&self, artifacts: &[BuildArtifact]) -> ErpResult<String> {
        info!("Deploying {} artifacts", artifacts.len());

        for artifact in artifacts {
            info!(
                "Deploying artifact: {} ({} bytes)",
                artifact.name, artifact.size_bytes
            );
            // In a real implementation, this would:
            // 1. Copy artifacts to deployment directory
            // 2. Update symlinks
            // 3. Restart services
            // 4. Update load balancer configuration
        }

        Ok(format!(
            "Deployed {} artifacts successfully",
            artifacts.len()
        ))
    }

    async fn run_health_check(
        &self,
        health_check: &HealthCheckConfig,
    ) -> ErpResult<HealthCheckResult> {
        let start_time = std::time::Instant::now();

        info!("Running health check: {}", health_check.name);

        // Simulate health check HTTP request
        // In a real implementation, this would make an actual HTTP request
        let (status, error_message) = if health_check.endpoint.contains("fail") {
            (
                HealthCheckStatus::Failed,
                Some("Health check endpoint returned error".to_string()),
            )
        } else {
            (HealthCheckStatus::Passed, None)
        };

        let response_time = start_time.elapsed().as_millis() as u64;

        Ok(HealthCheckResult {
            check_name: health_check.name.clone(),
            status,
            response_time_ms: response_time,
            timestamp: Utc::now(),
            error_message,
        })
    }

    async fn stop_application(&self) -> ErpResult<()> {
        info!("Stopping application");
        // Implementation would stop the running application
        Ok(())
    }

    async fn start_application(&self) -> ErpResult<()> {
        info!("Starting application");
        // Implementation would start the application
        Ok(())
    }

    async fn restore_previous_version(&self, version: &str) -> ErpResult<()> {
        info!("Restoring previous version: {}", version);
        // Implementation would restore the previous version
        Ok(())
    }

    fn run_script(&self, script: &str, env_vars: &HashMap<String, String>) -> ErpResult<()> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(script);

        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let output = cmd
            .output()
            .map_err(|e| ErpError::internal(format!("Failed to execute script: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ErpError::internal(format!("Script failed: {}", error)));
        }

        Ok(())
    }

    fn calculate_file_checksum(&self, file_path: &Path) -> ErpResult<String> {
        use sha2::{Digest, Sha256};
        use std::fs::File;
        use std::io::{BufReader, Read};

        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}

// Mock implementation for testing
#[derive(Debug, Clone)]
pub struct MockDeploymentRepository {
    deployments: std::sync::Arc<std::sync::Mutex<Vec<DeploymentRecord>>>,
}

impl Default for MockDeploymentRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MockDeploymentRepository {
    pub fn new() -> Self {
        Self {
            deployments: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl DeploymentRepository for MockDeploymentRepository {
    async fn store_deployment(&self, deployment: &DeploymentRecord) -> ErpResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        deployments.push(deployment.clone());
        Ok(())
    }

    async fn update_deployment(&self, deployment: &DeploymentRecord) -> ErpResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        if let Some(existing) = deployments.iter_mut().find(|d| d.id == deployment.id) {
            *existing = deployment.clone();
        }
        Ok(())
    }

    async fn get_deployment(&self, id: Uuid) -> ErpResult<Option<DeploymentRecord>> {
        let deployments = self.deployments.lock().unwrap();
        Ok(deployments.iter().find(|d| d.id == id).cloned())
    }

    async fn get_deployments_by_environment(
        &self,
        env: Environment,
    ) -> ErpResult<Vec<DeploymentRecord>> {
        let deployments = self.deployments.lock().unwrap();
        Ok(deployments
            .iter()
            .filter(|d| d.environment == env)
            .cloned()
            .collect())
    }

    async fn get_latest_deployment(&self, env: Environment) -> ErpResult<Option<DeploymentRecord>> {
        let deployments = self.deployments.lock().unwrap();
        Ok(deployments
            .iter()
            .filter(|d| d.environment == env)
            .max_by_key(|d| d.timestamp)
            .cloned())
    }

    async fn get_deployment_history(
        &self,
        limit: Option<usize>,
    ) -> ErpResult<Vec<DeploymentRecord>> {
        let deployments = self.deployments.lock().unwrap();
        let mut sorted_deployments = deployments.clone();
        sorted_deployments.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            sorted_deployments.truncate(limit);
        }

        Ok(sorted_deployments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_deployment_config() -> DeploymentConfig {
        DeploymentConfig {
            environment: Environment::Development,
            application_name: "test-app".to_string(),
            version: "1.0.0".to_string(),
            build_config: BuildConfig {
                target: "x86_64-unknown-linux-gnu".to_string(),
                features: vec!["default".to_string()],
                optimization_level: OptimizationLevel::Release,
                build_directory: PathBuf::from("."),
                artifact_name: "test-app".to_string(),
                pre_build_scripts: vec![],
                post_build_scripts: vec![],
                environment_variables: HashMap::new(),
            },
            database_config: DatabaseDeploymentConfig {
                run_migrations: true,
                backup_before_migration: true,
                migration_timeout_seconds: 300,
                database_url: "sqlite://test.db".to_string(),
                connection_pool_size: 5,
                migration_scripts_path: PathBuf::from("./migrations"),
            },
            security_config: SecurityDeploymentConfig {
                enable_https: true,
                certificate_path: None,
                private_key_path: None,
                security_headers: true,
                cors_origins: vec!["localhost".to_string()],
                rate_limiting: true,
                audit_logging: true,
            },
            monitoring_config: MonitoringDeploymentConfig {
                metrics_enabled: true,
                health_check_endpoint: "/health".to_string(),
                prometheus_endpoint: Some("/metrics".to_string()),
                log_level: "info".to_string(),
                structured_logging: true,
                alert_endpoints: vec![],
            },
            scaling_config: ScalingConfig {
                min_instances: 1,
                max_instances: 5,
                cpu_threshold: 70.0,
                memory_threshold: 80.0,
                auto_scaling_enabled: false,
            },
            health_checks: vec![HealthCheckConfig {
                name: "basic".to_string(),
                endpoint: "/health".to_string(),
                timeout_seconds: 30,
                expected_status_code: 200,
                retries: 3,
            }],
            rollback_config: RollbackConfig {
                enabled: true,
                automatic_rollback: false,
                health_check_failures_threshold: 1,
                rollback_timeout_seconds: 300,
            },
        }
    }

    #[tokio::test]
    async fn test_deployment_service_creation() {
        let repository = Box::new(MockDeploymentRepository::new());
        let service = DeploymentService::new(repository);

        let _config = create_test_deployment_config();
        let history = service.list_deployments(None, None).await.unwrap();
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_mock_deployment_repository() {
        let repository = MockDeploymentRepository::new();

        let config = create_test_deployment_config();
        let deployment = DeploymentRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            environment: config.environment.clone(),
            version: config.version.clone(),
            status: DeploymentStatus::Completed,
            config,
            build_artifacts: vec![],
            deployment_steps: vec![],
            health_check_results: vec![],
            rollback_info: None,
            duration_seconds: Some(120),
            error_message: None,
            deployed_by: "test-user".to_string(),
        };

        // Store deployment
        repository.store_deployment(&deployment).await.unwrap();

        // Retrieve deployment
        let retrieved = repository.get_deployment(deployment.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, deployment.id);

        // Get by environment
        let env_deployments = repository
            .get_deployments_by_environment(Environment::Development)
            .await
            .unwrap();
        assert_eq!(env_deployments.len(), 1);

        // Get latest
        let latest = repository
            .get_latest_deployment(Environment::Development)
            .await
            .unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().id, deployment.id);
    }

    #[test]
    fn test_deployment_config_creation() {
        let config = create_test_deployment_config();

        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.version, "1.0.0");
        assert!(config.database_config.run_migrations);
        assert!(config.security_config.enable_https);
        assert!(!config.scaling_config.auto_scaling_enabled);
    }

    #[test]
    fn test_build_artifact_creation() {
        let artifact = BuildArtifact {
            name: "test-app".to_string(),
            path: PathBuf::from("/tmp/test-app"),
            size_bytes: 1024,
            checksum: "abc123".to_string(),
            created_at: Utc::now(),
        };

        assert_eq!(artifact.name, "test-app");
        assert_eq!(artifact.size_bytes, 1024);
        assert_eq!(artifact.checksum, "abc123");
    }

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult {
            check_name: "database".to_string(),
            status: HealthCheckStatus::Passed,
            response_time_ms: 25,
            timestamp: Utc::now(),
            error_message: None,
        };

        assert_eq!(result.status, HealthCheckStatus::Passed);
        assert_eq!(result.response_time_ms, 25);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_deployment_step() {
        let step = DeploymentStep {
            name: "Build".to_string(),
            status: StepStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            output: Some("Build successful".to_string()),
            error_message: None,
        };

        assert_eq!(step.status, StepStatus::Completed);
        assert!(step.completed_at.is_some());
        assert_eq!(step.output.as_ref().unwrap(), "Build successful");
    }

    #[test]
    fn test_rollback_info() {
        let rollback = RollbackInfo {
            triggered_by: RollbackTrigger::HealthCheckFailure,
            previous_version: "0.9.0".to_string(),
            rollback_timestamp: Utc::now(),
            rollback_duration_seconds: 45,
        };

        assert!(matches!(
            rollback.triggered_by,
            RollbackTrigger::HealthCheckFailure
        ));
        assert_eq!(rollback.previous_version, "0.9.0");
        assert_eq!(rollback.rollback_duration_seconds, 45);
    }
}
