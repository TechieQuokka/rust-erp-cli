use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "erp")]
#[clap(about = "ERP CLI - Enterprise Resource Planning Command Line Interface")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// 설정 파일 경로 (선택사항)
    #[clap(long, global = true)]
    pub config: Option<String>,

    /// 로그 레벨 설정
    #[clap(long, global = true, value_enum)]
    pub log_level: Option<LogLevel>,

    /// 하위 명령어
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    /// 인벤토리 관리 명령어들
    #[clap(subcommand)]
    Inventory(InventoryCommands),
    /// 고객 관리 명령어들
    #[clap(subcommand)]
    Customers(CustomerCommands),
    /// 영업 관리 명령어들
    #[clap(subcommand)]
    Sales(SalesCommands),
    /// 보고서 명령어들
    #[clap(subcommand)]
    Reports(ReportCommands),
    /// 설정 관리 명령어들
    #[clap(subcommand)]
    Config(ConfigCommands),
    /// 데이터베이스 마이그레이션 명령어들
    #[clap(subcommand)]
    Migrate(MigrateCommands),
}

#[derive(Debug, clap::Subcommand)]
pub enum InventoryCommands {
    /// 제품 추가
    Add {
        /// 제품명
        name: String,
        /// 수량
        #[clap(long)]
        quantity: i32,
        /// 가격
        #[clap(long)]
        price: f64,
        /// 원가 (선택사항, 기본값: 가격의 70%)
        #[clap(long)]
        cost: Option<f64>,
        /// 카테고리
        #[clap(long)]
        category: Option<String>,
        /// SKU (선택사항)
        #[clap(long)]
        sku: Option<String>,
        /// 최소 재고량 (선택사항)
        #[clap(long)]
        min_stock: Option<i32>,
        /// 설명 (선택사항)
        #[clap(long)]
        description: Option<String>,
    },
    /// 제품 목록 조회
    List {
        /// 저재고 상품만 조회
        #[clap(long)]
        low_stock: bool,
        /// 카테고리 필터
        #[clap(long)]
        category: Option<String>,
        /// 제품명 또는 SKU 검색
        #[clap(long)]
        search: Option<String>,
        /// 페이지 번호
        #[clap(long, default_value = "1")]
        page: u32,
        /// 페이지당 아이템 수
        #[clap(long, default_value = "20")]
        limit: u32,
        /// 출력 형식 (table, json, csv)
        #[clap(long, default_value = "table")]
        format: String,
        /// 정렬 기준 (name, sku, quantity, price, cost, created_at)
        #[clap(long, default_value = "name")]
        sort_by: String,
        /// 정렬 순서 (asc, desc)
        #[clap(long, default_value = "asc")]
        order: String,
    },
    /// 제품 정보 수정
    Update {
        /// 제품 ID 또는 SKU
        id: String,
        /// 새로운 제품명
        #[clap(long)]
        name: Option<String>,
        /// 새로운 수량
        #[clap(long)]
        quantity: Option<i32>,
        /// 새로운 가격
        #[clap(long)]
        price: Option<f64>,
        /// 새로운 원가
        #[clap(long)]
        cost: Option<f64>,
        /// 새로운 카테고리
        #[clap(long)]
        category: Option<String>,
        /// 새로운 설명
        #[clap(long)]
        description: Option<String>,
    },
    /// 제품 삭제
    Remove {
        /// 제품 ID 또는 SKU
        id: String,
        /// 강제 삭제 (확인 없이)
        #[clap(long)]
        force: bool,
    },
    /// 저재고 상품 조회
    LowStock {
        /// 최소 재고량 임계값
        #[clap(long)]
        threshold: Option<i32>,
        /// 출력 형식 (table, json, csv)
        #[clap(long, default_value = "table", value_parser = ["table", "json", "csv"])]
        format: String,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum CustomerCommands {
    /// 고객 추가
    Add {
        /// 고객명 (전체 이름 또는 회사명)
        name: Option<String>,
        /// 성
        #[clap(long)]
        first_name: Option<String>,
        /// 이름
        #[clap(long)]
        last_name: Option<String>,
        /// 이메일
        #[clap(long)]
        email: String,
        /// 전화번호
        #[clap(long)]
        phone: Option<String>,
        /// 주소
        #[clap(long)]
        address: Option<String>,
        /// 회사명 (기업 고객인 경우 필수)
        #[clap(long)]
        company: Option<String>,
        /// 사업자등록번호 (기업 고객인 경우 필수)
        #[clap(long)]
        tax_id: Option<String>,
        /// 고객 관련 메모
        #[clap(long)]
        notes: Option<String>,
    },
    /// 고객 목록 조회
    List {
        /// 검색어
        #[clap(long)]
        search: Option<String>,
        /// 고객 타입 필터
        #[clap(long)]
        customer_type: Option<String>,
        /// 페이지 번호
        #[clap(long, default_value = "1")]
        page: u32,
        /// 페이지당 아이템 수
        #[clap(long, default_value = "20")]
        limit: u32,
        /// 출력 형식 (table, json, csv)
        #[clap(long, default_value = "table")]
        format: String,
        /// 정렬 기준 (name, email, created_at)
        #[clap(long, default_value = "name")]
        sort_by: String,
        /// 정렬 순서 (asc, desc)
        #[clap(long, default_value = "asc")]
        order: String,
    },
    /// 고객 정보 수정
    Update {
        /// 고객 ID
        id: String,
        /// 새로운 고객명
        #[clap(long)]
        name: Option<String>,
        /// 새로운 이메일
        #[clap(long)]
        email: Option<String>,
        /// 새로운 전화번호
        #[clap(long)]
        phone: Option<String>,
        /// 새로운 주소
        #[clap(long)]
        address: Option<String>,
        /// 새로운 회사명
        #[clap(long)]
        company: Option<String>,
        /// 새로운 메모
        #[clap(long)]
        notes: Option<String>,
    },
    /// 고객 삭제
    Delete {
        /// 고객 ID
        id: String,
        /// 강제 삭제 (확인 없이)
        #[clap(long)]
        force: bool,
        /// 관련된 모든 주문도 함께 삭제
        #[clap(long)]
        cascade: bool,
    },
    /// 고객 검색
    Search {
        /// 검색어
        query: String,
        /// 검색 필드 (name, email, phone)
        #[clap(long)]
        field: Option<String>,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum SalesCommands {
    /// 주문 생성
    CreateOrder {
        /// 고객 ID
        #[clap(long)]
        customer_id: String,
        /// 제품 SKU
        #[clap(long)]
        product_sku: String,
        /// 수량
        #[clap(long)]
        quantity: i32,
        /// 주문 메모
        #[clap(long)]
        notes: Option<String>,
    },
    /// 주문 목록 조회
    ListOrders {
        /// 주문 상태 필터
        #[clap(long)]
        status: Option<String>,
        /// 고객 ID 필터
        #[clap(long)]
        customer: Option<String>,
        /// 시작 날짜 (YYYY-MM-DD)
        #[clap(long)]
        from_date: Option<String>,
        /// 종료 날짜 (YYYY-MM-DD)
        #[clap(long)]
        to_date: Option<String>,
        /// 페이지 번호
        #[clap(long, default_value = "1")]
        page: u32,
        /// 페이지당 아이템 수
        #[clap(long, default_value = "20")]
        limit: u32,
        /// 출력 형식 (table, json, csv)
        #[clap(long, default_value = "table")]
        format: String,
    },
    /// 주문 상태 변경
    UpdateOrder {
        /// 주문 ID
        id: String,
        /// 새로운 상태 (pending, processing, shipped, delivered, cancelled)
        #[clap(long)]
        status: String,
        /// 상태 변경 메모
        #[clap(long)]
        notes: Option<String>,
    },
    /// 인보이스 생성
    GenerateInvoice {
        /// 주문 ID
        order_id: String,
        /// 출력 파일 경로
        #[clap(long)]
        output: Option<String>,
        /// 인보이스 형식 (pdf, html)
        #[clap(long, default_value = "pdf")]
        format: String,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum ReportCommands {
    /// 매출 요약 보고서
    SalesSummary {
        /// 기간 (daily, weekly, monthly, quarterly, yearly, custom)
        #[clap(long, default_value = "monthly")]
        period: String,
        /// 시작 날짜 (YYYY-MM-DD)
        #[clap(long)]
        from_date: Option<String>,
        /// 종료 날짜 (YYYY-MM-DD)
        #[clap(long)]
        to_date: Option<String>,
        /// 출력 형식 (console, json, csv, html, pdf)
        #[clap(long, default_value = "console")]
        format: String,
        /// 출력 파일 경로
        #[clap(long)]
        output: Option<String>,
    },
    /// 재고 상태 보고서
    InventoryStatus {
        /// 출력 형식 (console, table, json, csv, html, pdf)
        #[clap(long, default_value = "console")]
        format: String,
        /// 출력 파일 경로
        #[clap(long)]
        output: Option<String>,
        /// 특정 카테고리만 포함
        #[clap(long)]
        category: Option<String>,
        /// 저재고만 표시
        #[clap(long)]
        low_stock_only: bool,
        /// 저재고 기준 수량
        #[clap(long)]
        threshold: Option<u32>,
    },
    /// 고객 분석 보고서
    CustomerAnalysis {
        /// 분석 기간 (months)
        #[clap(long, default_value = "12")]
        months: u32,
        /// 출력 형식 (console, json, csv, html, pdf)
        #[clap(long, default_value = "console")]
        format: String,
        /// 출력 파일 경로
        #[clap(long)]
        output: Option<String>,
    },
    /// 재무 개요 보고서
    FinancialOverview {
        /// 시작 날짜 (YYYY-MM-DD)
        #[clap(long)]
        from_date: Option<String>,
        /// 종료 날짜 (YYYY-MM-DD)
        #[clap(long)]
        to_date: Option<String>,
        /// 기간 (daily, weekly, monthly, quarterly, yearly)
        #[clap(long)]
        period: Option<String>,
        /// 출력 형식 (table, console, json, csv, html, pdf)
        #[clap(long, default_value = "table")]
        format: String,
        /// 출력 파일 경로
        #[clap(long)]
        output: Option<String>,
        /// 차트 포함 여부
        #[clap(long)]
        include_charts: bool,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum ConfigCommands {
    /// 설정 조회
    Get {
        /// 설정 키
        key: String,
    },
    /// 설정 값 변경
    Set {
        /// 설정 키
        key: String,
        /// 설정 값
        value: String,
    },
    /// 설정 목록
    List {
        /// 필터 패턴
        #[clap(long)]
        filter: Option<String>,
        /// 출력 형식 (table, json, yaml)
        #[clap(long, default_value = "table")]
        format: String,
    },
    /// 설정 파일 경로 표시
    Path,
    /// 설정 초기화
    Reset {
        /// 강제 초기화 (확인 없이)
        #[clap(long)]
        force: bool,
    },
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum MigrateCommands {
    /// 데이터베이스 초기화 (처음 실행 시)
    Init {
        /// 강제 초기화 (기존 데이터 삭제)
        #[clap(long)]
        force: bool,
    },
    /// 마이그레이션 실행 (최신 상태로 업데이트)
    Up {
        /// 특정 버전까지만 실행
        #[clap(long)]
        target: Option<String>,
        /// 마이그레이션 파일 디렉토리
        #[clap(long, default_value = "migrations")]
        dir: String,
    },
    /// 마이그레이션 롤백
    Down {
        /// 롤백할 버전 (지정하지 않으면 1단계만)
        #[clap(long)]
        target: Option<String>,
        /// 마이그레이션 파일 디렉토리
        #[clap(long, default_value = "migrations")]
        dir: String,
    },
    /// 마이그레이션 상태 확인
    Status {
        /// 마이그레이션 파일 디렉토리
        #[clap(long, default_value = "migrations")]
        dir: String,
    },
    /// 새 마이그레이션 파일 생성
    Generate {
        /// 마이그레이션 이름
        name: String,
        /// 마이그레이션 파일 디렉토리
        #[clap(long, default_value = "migrations")]
        dir: String,
    },
    /// 데이터베이스 연결 테스트
    Test,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn try_parse_args() -> Result<Self, clap::Error> {
        Self::try_parse()
    }
}
