use clap::Parser;
use erp_cli::cli::parser::Cli;

#[tokio::main]
async fn main() {
    // CLI 파싱만 테스트 (Phase 3)
    let cli = Cli::parse();

    // 기본 설정 생성 (Phase 3에서는 간단한 더미 설정 사용)
    let config = erp_cli::core::config::AppConfig::default();

    // CLI 실행
    if let Err(e) = cli.run(config).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
