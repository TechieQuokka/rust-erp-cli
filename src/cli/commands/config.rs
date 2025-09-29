use crate::cli::parser::ConfigCommands;
use crate::core::config::AppConfig;
use crate::core::database::DatabaseConnection;
use crate::modules::config::{
    ConfigFilter, ConfigItem, ConfigRepository, ConfigService, CreateConfigRequest,
    UpdateConfigRequest,
};
use crate::utils::error::ErpResult;
use console::{style, Term};
use std::sync::Arc;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ConfigItemDisplay {
    #[tabled(rename = "Key")]
    key: String,
    #[tabled(rename = "Value")]
    value: String,
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Secret")]
    is_secret: String,
    #[tabled(rename = "Readonly")]
    is_readonly: String,
    #[tabled(rename = "Description")]
    description: String,
}

impl From<ConfigItem> for ConfigItemDisplay {
    fn from(item: ConfigItem) -> Self {
        let value = if item.is_secret {
            item.masked_value()
        } else {
            item.value.clone()
        };

        Self {
            key: item.key,
            value,
            category: item.category,
            is_secret: if item.is_secret {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
            is_readonly: if item.is_readonly {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
            description: item.description.unwrap_or_else(|| "-".to_string()),
        }
    }
}

pub struct ConfigHandler;

impl ConfigHandler {
    pub async fn handle(cmd: &ConfigCommands, config: &AppConfig) -> ErpResult<()> {
        // 데이터베이스 연결 및 서비스 초기화
        let db_conn = Arc::new(DatabaseConnection::new(config.database.clone()).await?);
        let repository = Arc::new(ConfigRepository::new(db_conn.clone()));

        // 테이블 초기화 (존재하지 않는 경우)
        repository.init_table().await?;

        let service = ConfigService::new(repository);

        match cmd {
            ConfigCommands::Get { key } => Self::handle_get(key, &service).await,
            ConfigCommands::Set { key, value } => Self::handle_set(key, value, &service).await,
            ConfigCommands::List { filter, format } => {
                Self::handle_list(filter, format, &service).await
            }
            ConfigCommands::Path => Self::handle_path().await,
            ConfigCommands::Reset { force } => Self::handle_reset(*force, &service).await,
        }
    }

    async fn handle_get(key: &str, service: &ConfigService) -> ErpResult<()> {
        let term = Term::stdout();

        match service.get_config(key).await? {
            Some(config) => {
                term.write_line(&format!(
                    "{}: {}",
                    style("Key").cyan().bold(),
                    style(&config.key).white()
                ))
                .ok();

                term.write_line(&format!(
                    "{}: {}",
                    style("Value").cyan().bold(),
                    if config.is_secret {
                        style(config.masked_value()).red().to_string()
                    } else {
                        style(config.value.clone()).green().to_string()
                    }
                ))
                .ok();

                term.write_line(&format!(
                    "{}: {}",
                    style("Category").cyan().bold(),
                    style(&config.category).yellow()
                ))
                .ok();

                if let Some(desc) = &config.description {
                    term.write_line(&format!(
                        "{}: {}",
                        style("Description").cyan().bold(),
                        style(desc).white()
                    ))
                    .ok();
                }

                if config.is_secret {
                    term.write_line(&style("🔐 This is a secret configuration").red().to_string())
                        .ok();
                }

                if config.is_readonly {
                    term.write_line(
                        &style("🔒 This configuration is read-only")
                            .yellow()
                            .to_string(),
                    )
                    .ok();
                }
            }
            None => {
                term.write_line(
                    &style(format!("Configuration '{}' not found", key))
                        .red()
                        .to_string(),
                )
                .ok();
            }
        }

        Ok(())
    }

    async fn handle_set(key: &str, value: &str, service: &ConfigService) -> ErpResult<()> {
        let term = Term::stdout();

        // 기존 설정이 있는지 확인
        match service.get_config(key).await? {
            Some(_existing) => {
                // 기존 설정 업데이트
                let update_request = UpdateConfigRequest {
                    value: Some(value.to_string()),
                    description: None,
                    category: None,
                    is_secret: None,
                };

                let updated = service.update_config(key, update_request).await?;

                term.write_line(&format!(
                    "✅ {}: {}",
                    style("Updated configuration").green().bold(),
                    style(&updated.key).white()
                ))
                .ok();

                term.write_line(&format!(
                    "{}: {}",
                    style("New Value").cyan(),
                    if updated.is_secret {
                        style(updated.masked_value()).red().to_string()
                    } else {
                        style(updated.value.clone()).green().to_string()
                    }
                ))
                .ok();
            }
            None => {
                // 새로운 설정 생성 (기본 카테고리 'system' 사용)
                let create_request = CreateConfigRequest::new(
                    key.to_string(),
                    value.to_string(),
                    "system".to_string(),
                );

                let created = service.create_config(create_request).await?;

                term.write_line(&format!(
                    "✅ {}: {}",
                    style("Created new configuration").green().bold(),
                    style(&created.key).white()
                ))
                .ok();

                term.write_line(&format!(
                    "{}: {}",
                    style("Value").cyan(),
                    style(&created.value).green()
                ))
                .ok();

                term.write_line(&format!(
                    "{}: {}",
                    style("Category").cyan(),
                    style(&created.category).yellow()
                ))
                .ok();
            }
        }

        Ok(())
    }

    async fn handle_list(
        filter: &Option<String>,
        _format: &str,
        service: &ConfigService,
    ) -> ErpResult<()> {
        let term = Term::stdout();

        // 필터 생성
        let config_filter = if let Some(pattern) = filter {
            if !pattern.trim().is_empty() {
                Some(ConfigFilter::new().with_key_pattern(pattern.trim().to_string()))
            } else {
                None
            }
        } else {
            None
        };

        // 설정 목록 조회 (표시용으로 마스킹된 버전)
        let configs = service.list_configs_for_display(config_filter).await?;

        if configs.is_empty() {
            term.write_line(&style("No configurations found.").yellow().to_string())
                .ok();
            return Ok(());
        }

        // 테이블로 출력
        let display_items: Vec<ConfigItemDisplay> =
            configs.into_iter().map(ConfigItemDisplay::from).collect();

        let table = Table::new(&display_items);
        term.write_line(&table.to_string()).ok();

        // 통계 정보 출력
        let stats = service.get_config_statistics().await?;
        term.write_line("")?; // 빈 줄
        term.write_line(&format!(
            "{}: {}",
            style("Total configurations").cyan().bold(),
            style(stats.total_count).white()
        ))
        .ok();

        term.write_line(&format!(
            "{}: {}",
            style("Categories").cyan().bold(),
            style(stats.category_count).white()
        ))
        .ok();

        if stats.secret_count > 0 {
            term.write_line(&format!(
                "{}: {}",
                style("Secret configurations").red().bold(),
                style(stats.secret_count).white()
            ))
            .ok();
        }

        if stats.readonly_count > 0 {
            term.write_line(&format!(
                "{}: {}",
                style("Read-only configurations").yellow().bold(),
                style(stats.readonly_count).white()
            ))
            .ok();
        }

        Ok(())
    }

    async fn handle_path() -> ErpResult<()> {
        let term = Term::stdout();

        term.write_line(&style("Configuration File Paths:").cyan().bold().to_string())
            .ok();
        term.write_line("")?; // 빈 줄

        let current_dir = std::env::current_dir().map_err(|e| {
            crate::utils::error::ErpError::internal(format!(
                "Failed to get current directory: {}",
                e
            ))
        })?;

        term.write_line(&format!(
            "{}: {}",
            style("Current Directory").yellow().bold(),
            style(current_dir.display().to_string()).white()
        ))
        .ok();

        term.write_line("")?; // 빈 줄

        // 설정 파일들 확인
        let config_files = vec![
            ("System Config (default.toml)", "config/default.toml"),
            ("Development Config", "config/development.toml"),
            ("Production Config", "config/production.toml"),
        ];

        for (name, path) in config_files {
            let full_path = current_dir.join(path);
            let exists = full_path.exists();

            term.write_line(&format!(
                "{}: {}",
                if exists {
                    style(format!("✅ {}", name)).green()
                } else {
                    style(format!("❌ {}", name)).red()
                },
                style(full_path.display()).white()
            ))
            .ok();
        }

        term.write_line("")?; // 빈 줄

        // 데이터베이스 파일 경로
        if let Ok(db_path) = std::env::var("DATABASE_URL") {
            term.write_line(&format!(
                "{}: {}",
                style("Database").cyan().bold(),
                style(&db_path).white()
            ))
            .ok();
        } else {
            term.write_line(&format!(
                "{}: {}",
                style("Database").cyan().bold(),
                style("Using default SQLite database").white()
            ))
            .ok();
        }

        Ok(())
    }

    async fn handle_reset(force: bool, service: &ConfigService) -> ErpResult<()> {
        let term = Term::stdout();

        if !force {
            term.write_line(
                &style("⚠️  Configuration reset requires --force flag")
                    .red()
                    .bold()
                    .to_string(),
            )
            .ok();
            term.write_line(
                &style("This will delete all non-readonly configurations")
                    .yellow()
                    .to_string(),
            )
            .ok();
            term.write_line(&style("Use: config reset --force").white().to_string())
                .ok();
            return Ok(());
        }

        // 확인 메시지
        term.write_line(
            &style("🗑️  Resetting all non-readonly configurations...")
                .yellow()
                .bold()
                .to_string(),
        )
        .ok();

        // 초기화 실행
        let deleted_count = service.reset_configs(force).await?;

        if deleted_count > 0 {
            term.write_line(&format!(
                "✅ {} configurations were deleted",
                style(deleted_count).green().bold()
            ))
            .ok();
        } else {
            term.write_line(
                &style("No configurations were deleted (all are read-only)")
                    .yellow()
                    .to_string(),
            )
            .ok();
        }

        Ok(())
    }
}
