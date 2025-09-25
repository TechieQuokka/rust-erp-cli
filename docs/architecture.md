# Rust ERP CLI 시스템 아키텍처 설계

## 1. 개요

### 1.1 목적
- Rust 기반의 고성능 ERP CLI 시스템 구축
- 모듈형 아키텍처로 확장 가능한 시스템 설계
- 명령줄 인터페이스를 통한 직관적인 ERP 기능 제공

### 1.2 주요 특징
- **고성능**: Rust의 메모리 안전성과 성능 활용
- **모듈성**: 독립적인 모듈로 구성된 아키텍처
- **확장성**: 새로운 기능 모듈 쉽게 추가 가능
- **보안성**: 강력한 인증 및 데이터 보호 기능
- **사용성**: 직관적인 CLI 인터페이스

## 2. 시스템 아키텍처

### 2.1 전체 아키텍처 다이어그램

```
┌─────────────────────────────────────────────────────────────┐
│                     ERP CLI System                         │
├─────────────────────────────────────────────────────────────┤
│  CLI Interface Layer                                       │
│  ┌─────────────────┬─────────────────┬─────────────────┐   │
│  │   Commands      │    Parser       │    Validator    │   │
│  └─────────────────┴─────────────────┴─────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  Business Logic Layer                                      │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐ │
│  │Inventory │  Sales   │Customers │ Reports  │  Config  │ │
│  │ Module   │  Module  │  Module  │  Module  │  Module  │ │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Core Services Layer                                       │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │ Auth Service│Database Svc │Config Svc  │  Log Service│ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Data Layer                                                │
│  ┌─────────────────┬─────────────────┬─────────────────┐   │
│  │   PostgreSQL    │     SQLite      │     Redis       │   │
│  │   (Production)  │   (Development) │   (Caching)     │   │
│  └─────────────────┴─────────────────┴─────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 프로젝트 구조

```
erp-cli/
├── Cargo.toml                 # 프로젝트 설정
├── README.md                  # 프로젝트 설명
├── LICENSE                    # 라이센스
│
├── src/
│   ├── main.rs               # 애플리케이션 진입점
│   ├── lib.rs                # 라이브러리 루트
│   │
│   ├── cli/                  # CLI 인터페이스 레이어
│   │   ├── mod.rs
│   │   ├── commands/         # 명령어 정의
│   │   │   ├── mod.rs
│   │   │   ├── inventory.rs
│   │   │   ├── sales.rs
│   │   │   ├── customers.rs
│   │   │   ├── reports.rs
│   │   │   └── config.rs
│   │   ├── parser.rs         # 명령어 파싱
│   │   └── validator.rs      # 입력 검증
│   │
│   ├── modules/              # 비즈니스 로직 모듈
│   │   ├── mod.rs
│   │   ├── inventory/        # 재고 관리 모듈
│   │   │   ├── mod.rs
│   │   │   ├── service.rs
│   │   │   ├── models.rs
│   │   │   └── repository.rs
│   │   ├── sales/            # 영업 관리 모듈
│   │   │   ├── mod.rs
│   │   │   ├── service.rs
│   │   │   ├── models.rs
│   │   │   └── repository.rs
│   │   ├── customers/        # 고객 관리 모듈
│   │   │   ├── mod.rs
│   │   │   ├── service.rs
│   │   │   ├── models.rs
│   │   │   └── repository.rs
│   │   ├── reports/          # 보고서 모듈
│   │   │   ├── mod.rs
│   │   │   ├── service.rs
│   │   │   └── generators.rs
│   │   └── config/           # 설정 관리 모듈
│   │       ├── mod.rs
│   │       └── service.rs
│   │
│   ├── core/                 # 핵심 서비스
│   │   ├── mod.rs
│   │   ├── auth/             # 인증 서비스
│   │   │   ├── mod.rs
│   │   │   ├── service.rs
│   │   │   ├── jwt.rs
│   │   │   └── rbac.rs
│   │   ├── database/         # 데이터베이스 서비스
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs
│   │   │   ├── migration.rs
│   │   │   └── models/
│   │   │       ├── mod.rs
│   │   │       ├── user.rs
│   │   │       ├── product.rs
│   │   │       ├── customer.rs
│   │   │       └── order.rs
│   │   ├── config/           # 설정 서비스
│   │   │   ├── mod.rs
│   │   │   └── loader.rs
│   │   └── logging/          # 로깅 서비스
│   │       ├── mod.rs
│   │       └── logger.rs
│   │
│   ├── utils/                # 유틸리티
│   │   ├── mod.rs
│   │   ├── error.rs          # 에러 타입 정의
│   │   ├── crypto.rs         # 암호화 유틸리티
│   │   └── validation.rs     # 검증 유틸리티
│   │
│   └── api/                  # REST API (선택적)
│       ├── mod.rs
│       ├── handlers/
│       │   ├── mod.rs
│       │   ├── inventory.rs
│       │   ├── sales.rs
│       │   └── customers.rs
│       └── routes.rs
│
├── migrations/               # 데이터베이스 마이그레이션
│   ├── 001_initial.sql
│   ├── 002_add_customers.sql
│   └── 003_add_inventory.sql
│
├── config/                   # 설정 파일
│   ├── default.toml         # 기본 설정
│   ├── development.toml     # 개발 환경 설정
│   └── production.toml      # 프로덕션 환경 설정
│
├── docs/                     # 문서
│   ├── architecture.md      # 이 파일
│   ├── api-reference.md     # API 참조
│   ├── user-guide.md        # 사용자 가이드
│   └── development.md       # 개발 가이드
│
└── tests/                    # 테스트
    ├── integration/         # 통합 테스트
    ├── unit/               # 단위 테스트
    └── fixtures/           # 테스트 데이터
```

## 3. 핵심 모듈 설계

### 3.1 재고 관리 (Inventory Module)

**주요 기능:**
- 제품 정보 관리 (추가, 수정, 삭제, 조회)
- 재고 수량 추적
- 저재고 알림
- 재고 이동 기록

**CLI 명령어:**
```bash
# 제품 추가
erp inventory add "제품명" --quantity 100 --price 10000 --category "전자제품"

# 재고 조회
erp inventory list --low-stock --category "전자제품"

# 재고 업데이트
erp inventory update PRODUCT_ID --quantity 50

# 제품 삭제
erp inventory remove PRODUCT_ID
```

**데이터 모델:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sku: String,
    pub category: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### 3.2 영업 관리 (Sales Module)

**주요 기능:**
- 주문 생성 및 관리
- 인보이스 생성
- 주문 상태 추적
- 매출 통계

**CLI 명령어:**
```bash
# 주문 생성
erp sales create-order --customer CUSTOMER_ID --product PRODUCT_ID --quantity 5

# 주문 목록 조회
erp sales list-orders --status pending --date-range "2024-01-01,2024-01-31"

# 주문 상태 업데이트
erp sales update-order ORDER_ID --status completed

# 인보이스 생성
erp sales generate-invoice ORDER_ID
```

### 3.3 고객 관리 (Customer Module)

**주요 기능:**
- 고객 정보 관리
- 고객 검색 및 필터링
- 고객 거래 이력 조회

**CLI 명령어:**
```bash
# 고객 추가
erp customers add "고객명" --email "customer@example.com" --phone "010-1234-5678"

# 고객 목록 조회
erp customers list --search "고객명"

# 고객 정보 수정
erp customers update CUSTOMER_ID --email "new@example.com"

# 고객 삭제
erp customers delete CUSTOMER_ID
```

### 3.4 보고서 (Reports Module)

**주요 기능:**
- 매출 요약 보고서
- 재고 상태 보고서
- 고객 분석 보고서
- 재무 개요

**CLI 명령어:**
```bash
# 매출 요약
erp reports sales-summary --period monthly --year 2024

# 재고 상태
erp reports inventory-status

# 고객 분석
erp reports customer-analysis --top 10

# 재무 개요
erp reports financial-overview --quarter Q1
```

## 4. 데이터베이스 설계

### 4.1 주요 테이블

**Users (사용자)**
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

**Products (제품)**
```sql
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sku VARCHAR(100) UNIQUE NOT NULL,
    category VARCHAR(100) NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    unit_price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

**Customers (고객)**
```sql
CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(20),
    address TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

**Orders (주문)**
```sql
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    total_amount DECIMAL(10,2) NOT NULL,
    order_date TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

**Order Items (주문 항목)**
```sql
CREATE TABLE order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id),
    product_id UUID NOT NULL REFERENCES products(id),
    quantity INTEGER NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    total_price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 4.2 인덱스 전략

```sql
-- 성능 최적화를 위한 인덱스
CREATE INDEX idx_products_category ON products(category);
CREATE INDEX idx_products_sku ON products(sku);
CREATE INDEX idx_orders_customer_id ON orders(customer_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_date ON orders(order_date);
CREATE INDEX idx_order_items_order_id ON order_items(order_id);
CREATE INDEX idx_order_items_product_id ON order_items(product_id);
```

## 5. 기술 스택

### 5.1 핵심 라이브러리

```toml
[dependencies]
# CLI 관련
clap = { version = "4.0", features = ["derive"] }
console = "0.15"
indicatif = "0.17"

# 비동기 처리
tokio = { version = "1.0", features = ["full"] }

# 데이터베이스
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "sqlite", "uuid", "chrono", "decimal"] }

# 직렬화/역직렬화
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 설정 관리
config = "0.14"
toml = "0.8"

# 에러 처리
thiserror = "1.0"
anyhow = "1.0"

# 로깅
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 날짜/시간
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.0", features = ["v4", "serde"] }

# 암호화
bcrypt = "0.14"
jsonwebtoken = "9.0"

# 소수점 연산
rust_decimal = { version = "1.0", features = ["serde"] }

# HTTP 클라이언트 (API 통신용)
reqwest = { version = "0.11", features = ["json"] }

# 테스트
[dev-dependencies]
tempfile = "3.0"
mockall = "0.11"
```

### 5.2 개발 도구

```toml
# 코드 품질
cargo-clippy = "latest"
cargo-fmt = "latest"

# 테스트
cargo-nextest = "latest"
cargo-tarpaulin = "latest"  # 코드 커버리지

# 문서화
cargo-doc = "latest"

# 성능 프로파일링
flamegraph = "latest"
```

## 6. 보안 설계

### 6.1 인증 및 권한 관리

**JWT 기반 인증:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Manager,
    User,
    ReadOnly,
}
```

**RBAC (Role-Based Access Control):**
```rust
pub struct Permission {
    pub resource: String,
    pub action: String,
}

pub trait AuthorizedUser {
    fn can(&self, permission: &Permission) -> bool;
    fn has_role(&self, role: UserRole) -> bool;
}
```

### 6.2 데이터 보호

- **비밀번호 해싱**: bcrypt 알고리즘 사용
- **민감한 설정 암호화**: AES-256-GCM 사용
- **SQL Injection 방지**: Prepared Statements 사용
- **입력 검증**: 모든 사용자 입력에 대한 검증

### 6.3 감사 로그

```rust
#[derive(Debug, Serialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

## 7. 설정 관리

### 7.1 설정 파일 구조

**config/default.toml:**
```toml
[database]
# SQLite for development, PostgreSQL for production
url = "sqlite://erp.db"
max_connections = 10
migrate_on_start = true

[logging]
level = "info"
format = "json"
file = "logs/erp.log"
rotate_daily = true

[auth]
jwt_secret = "your-super-secret-key-here"
token_expiry_hours = 24
password_min_length = 8

[cache]
redis_url = "redis://localhost:6379"
default_ttl_seconds = 3600

[api]
host = "127.0.0.1"
port = 8080
cors_enabled = true

[reports]
output_dir = "reports"
default_format = "json"
```

### 7.2 환경별 설정

**개발 환경 (config/development.toml):**
```toml
[database]
url = "sqlite://dev.db"

[logging]
level = "debug"
format = "pretty"

[auth]
jwt_secret = "dev-secret"
```

**프로덕션 환경 (config/production.toml):**
```toml
[database]
url = "${DATABASE_URL}"
max_connections = 20

[logging]
level = "warn"
format = "json"

[auth]
jwt_secret = "${JWT_SECRET}"
```

## 8. 에러 처리 전략

### 8.1 커스텀 에러 타입

```rust
#[derive(Debug, thiserror::Error)]
pub enum ErpError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Authentication failed: {reason}")]
    Auth { reason: String },

    #[error("Authorization failed: user {user_id} cannot {action} on {resource}")]
    Forbidden {
        user_id: Uuid,
        action: String,
        resource: String,
    },

    #[error("Validation error: {field} is {reason}")]
    Validation { field: String, reason: String },

    #[error("Resource not found: {resource_type} with id {id}")]
    NotFound {
        resource_type: String,
        id: String,
    },

    #[error("Business rule violation: {rule}")]
    BusinessRule { rule: String },

    #[error("External service error: {service} returned {status}")]
    ExternalService { service: String, status: String },
}

pub type ErpResult<T> = Result<T, ErpError>;
```

### 8.2 에러 처리 미들웨어

```rust
pub trait ErrorHandler {
    fn handle_error(&self, error: ErpError) -> ErpError;
    fn should_retry(&self, error: &ErpError) -> bool;
    fn get_retry_count(&self, error: &ErpError) -> u32;
}
```

## 9. 성능 최적화

### 9.1 데이터베이스 최적화

- **연결 풀링**: SQLx의 연결 풀 활용
- **쿼리 최적화**: N+1 문제 해결
- **인덱싱**: 자주 검색되는 필드에 인덱스 생성
- **페이지네이션**: 대량 데이터 조회 시 페이지네이션 적용

### 9.2 캐싱 전략

```rust
pub trait CacheService {
    async fn get<T>(&self, key: &str) -> ErpResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>;

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> ErpResult<()>
    where
        T: Serialize;

    async fn delete(&self, key: &str) -> ErpResult<()>;

    async fn clear_pattern(&self, pattern: &str) -> ErpResult<()>;
}
```

### 9.3 메모리 관리

- **Lazy Loading**: 필요한 시점에 데이터 로드
- **String Interning**: 반복되는 문자열 최적화
- **Arc/Rc 활용**: 데이터 공유 시 참조 카운팅 사용

## 10. 테스트 전략

### 10.1 테스트 피라미드

```rust
// 단위 테스트
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_create_product_success() {
        // Given
        let mut mock_repo = MockProductRepository::new();
        mock_repo.expect_create()
            .with(eq(product_data))
            .times(1)
            .returning(|_| Ok(mock_product()));

        let service = ProductService::new(Arc::new(mock_repo));

        // When
        let result = service.create_product(product_data).await;

        // Then
        assert!(result.is_ok());
    }
}

// 통합 테스트
#[tokio::test]
async fn test_full_order_workflow() {
    let test_db = setup_test_database().await;
    let app = create_test_app(test_db).await;

    // 1. 고객 생성
    let customer = app.create_customer(customer_data).await?;

    // 2. 제품 생성
    let product = app.create_product(product_data).await?;

    // 3. 주문 생성
    let order = app.create_order(order_data).await?;

    // 4. 검증
    assert_eq!(order.status, OrderStatus::Pending);
    assert_eq!(order.total_amount, expected_total);
}
```

### 10.2 테스트 데이터 관리

```rust
pub struct TestFixtures;

impl TestFixtures {
    pub fn mock_product() -> Product {
        Product {
            id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            sku: "TEST001".to_string(),
            category: "Test Category".to_string(),
            quantity: 100,
            unit_price: Decimal::from(1000),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            description: None,
        }
    }

    pub fn mock_customer() -> Customer {
        // Similar implementation
    }
}
```

## 11. 배포 및 운영

### 11.1 빌드 최적화

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 11.2 Docker 컨테이너화

```dockerfile
# Multi-stage build for optimal image size
FROM rust:1.75-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/erp-cli /usr/local/bin/
ENTRYPOINT ["erp-cli"]
```

### 11.3 모니터링

```rust
// 메트릭 수집
pub struct Metrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub database_connections: Gauge,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
}

// 헬스 체크
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub database: ComponentHealth,
    pub cache: ComponentHealth,
    pub version: String,
    pub uptime: Duration,
}
```

## 12. 확장 계획

### 12.1 웹 인터페이스 추가

- REST API를 활용한 웹 대시보드 구축
- React 또는 Vue.js 기반 프론트엔드
- 실시간 데이터 업데이트 (WebSocket)

### 12.2 모바일 앱 지원

- REST API 기반 모바일 앱 개발
- 오프라인 동기화 지원
- 푸시 알림 기능

### 12.3 마이크로서비스 아키텍처

- 각 모듈을 독립적인 서비스로 분리
- gRPC 기반 서비스 간 통신
- Kubernetes 기반 오케스트레이션

### 12.4 AI/ML 기능

- 수요 예측 모델
- 재고 최적화 알고리즘
- 고객 세그멘테이션

## 13. 결론

본 아키텍처는 Rust의 장점을 최대한 활용하여 고성능, 안전성, 확장성을 갖춘 ERP CLI 시스템을 구축하는 것을 목표로 합니다. 모듈형 아키텍처를 통해 기능별로 독립적인 개발과 유지보수가 가능하며, 향후 웹 인터페이스나 모바일 앱으로의 확장도 용이하게 설계되었습니다.

핵심 설계 원칙:
1. **모듈성**: 각 기능을 독립적인 모듈로 구성
2. **확장성**: 새로운 기능 추가가 용이한 구조
3. **보안성**: 강력한 인증/인가 및 데이터 보호
4. **성능**: 효율적인 데이터베이스 설계 및 캐싱
5. **유지보수성**: 명확한 코드 구조와 테스트 전략

이 아키텍처를 바탕으로 점진적인 개발을 진행하여 완성도 높은 ERP 시스템을 구축할 수 있을 것입니다.