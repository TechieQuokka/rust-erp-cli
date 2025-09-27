use clap::Parser;
use erp_cli::cli::parser::Cli;

#[tokio::main]
async fn main() {
    // .env 파일 로드
    dotenvy::dotenv().ok();

    // CLI 파싱
    let cli = Cli::parse();

    // 실제 설정 로드
    let config = match erp_cli::core::config::AppConfig::load().await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // CLI 실행
    if let Err(e) = cli.run(config).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
