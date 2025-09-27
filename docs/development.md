# ERP CLI 개발자 가이드

## 프로젝트 개요

ERP CLI는 4-레이어 아키텍처를 따르는 Rust 기반 모듈형 ERP(Enterprise Resource Planning) CLI 시스템입니다.

### 아키텍처 레이어

1. **CLI Interface Layer** - 명령어 파싱, 검증, 사용자 상호작용
2. **Business Logic Layer** - 핵심 비즈니스 모듈 (inventory, sales, customers, reports, config)
3. **Core Services Layer** - 인증, 데이터베이스, 설정, 로깅 서비스
4. **Data Layer** - PostgreSQL (프로덕션), SQLite (개발), Redis (캐싱)

## 개발 환경 설정

### 요구사항

- Rust 1.70 이상
- PostgreSQL 13 이상 (프로덕션용)
- SQLite (개발용)
- Redis (선택사항, 캐싱용)

### 프로젝트 설정

```bash
# 프로젝트 클론
git clone <repository-url>
cd erp

# 의존성 설치 및 빌드
cargo build

# 개발 환경 설정
cp .env.example .env
# .env 파일을 편집하여 데이터베이스 URL 등 설정

# 개발용 데이터베이스 초기화
cargo run -- setup --init-db
```

## 개발 명령어

### 빌드 및 실행

```bash
# 개발 빌드
cargo build

# 릴리스 빌드
cargo build --release

# CLI 실행
./target/release/erp --help
cargo run -- --help

# 특정 명령어 테스트
cargo run -- inventory list
```

### 테스트

```bash
# 모든 테스트 실행
cargo test

# nextest 사용 (설치된 경우)
cargo nextest run

# 특정 모듈 테스트
cargo test inventory

# 테스트 커버리지 (cargo-tarpaulin 필요)
cargo tarpaulin --out Html
```

### 코드 품질

```bash
# 코드 포맷팅
cargo fmt

# Clippy 린트 실행
cargo clippy -- -D warnings

# 빌드 없이 코드 체크
cargo check

# 문서 생성
cargo doc --open
```

## 프로젝트 구조

```
src/
├── cli/                    # CLI 인터페이스 레이어
│   ├── commands/          # 명령어 정의
│   ├── parser.rs          # clap 기반 파서
│   ├── validator.rs       # CLI 입력 검증
│   └── mod.rs
├── modules/               # 비즈니스 로직 레이어
│   ├── inventory/         # 재고 관리
│   ├── sales/             # 영업 관리
│   ├── customers/         # 고객 관리
│   ├── reports/           # 보고서
│   ├── config/            # 설정 관리
│   └── mod.rs
├── core/                  # 코어 서비스 레이어
│   ├── auth/              # 인증 시스템
│   ├── database/          # 데이터베이스 연결 및 모델
│   ├── config/            # 설정 로딩
│   ├── logging/           # 구조화된 로깅
│   ├── security/          # 보안 시스템
│   ├── ops/               # 운영 도구
│   └── mod.rs
├── utils/                 # 공유 유틸리티
│   ├── error.rs           # 에러 처리
│   ├── crypto.rs          # 암호화
│   ├── validation.rs      # 입력 검증
│   └── mod.rs
├── main.rs               # 메인 엔트리포인트
└── lib.rs                # 라이브러리 루트
```

## 핵심 설계 패턴

### Repository Pattern

각 모듈은 데이터 접근을 위한 repository를 가집니다:

```rust
// modules/inventory/repository.rs
#[async_trait]
pub trait InventoryRepository {
    async fn create_product(&self, product: &NewProduct) -> ErpResult<Product>;
    async fn find_by_sku(&self, sku: &str) -> ErpResult<Option<Product>>;
    async fn list_products(&self, filter: &ProductFilter) -> ErpResult<Vec<Product>>;
    // ...
}

pub struct SqlxInventoryRepository {
    pool: Arc<PgPool>,
}

#[async_trait]
impl InventoryRepository for SqlxInventoryRepository {
    // 구현...
}
```

### Service Layer

비즈니스 로직은 서비스 레이어에서 분리됩니다:

```rust
// modules/inventory/service.rs
pub struct InventoryService {
    repository: Arc<dyn InventoryRepository + Send + Sync>,
}

impl InventoryService {
    pub async fn add_product(&self, request: AddProductRequest) -> ErpResult<Product> {
        // 비즈니스 로직 검증
        self.validate_product_data(&request)?;

        // Repository를 통한 데이터 저장
        let new_product = NewProduct::from(request);
        self.repository.create_product(&new_product).await
    }
}
```

### Command Pattern

CLI 명령어는 clap derive 매크로를 사용하여 구조화됩니다:

```rust
// cli/commands/inventory.rs
#[derive(Parser)]
pub enum InventoryCommand {
    Add(AddProductArgs),
    List(ListProductsArgs),
    Update(UpdateProductArgs),
    Remove(RemoveProductArgs),
}

#[derive(Args)]
pub struct AddProductArgs {
    #[arg(long)]
    pub name: String,

    #[arg(long)]
    pub sku: String,

    #[arg(long)]
    pub quantity: u32,
    // ...
}
```

### Error Handling

커스텀 에러 타입을 사용하여 일관된 에러 처리를 제공합니다:

```rust
// utils/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ErpError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Authentication failed: {reason}")]
    Authentication { reason: String },
    // ...
}

pub type ErpResult<T> = Result<T, ErpError>;
```

## 새로운 기능 추가하기

### 1. 새로운 CLI 명령어 추가

1. `src/cli/commands/` 에서 적절한 파일에 명령어 구조체 추가
2. 해당 비즈니스 모듈에서 명령어 핸들러 구현
3. `src/cli/commands/mod.rs` 에서 명령어 등록
4. 모듈의 테스트 파일에 테스트 추가

### 2. 새로운 비즈니스 모듈 추가

1. `src/modules/` 하위에 모듈 디렉토리 생성
2. 다음 파일들 구현:
   - `models.rs` - 데이터 모델
   - `repository.rs` - 데이터 접근 레이어
   - `service.rs` - 비즈니스 로직
   - `mod.rs` - 모듈 정의
3. `src/core/database/models/` 에 데이터베이스 모델 추가
4. 필요시 마이그레이션 파일 생성
5. `src/modules/mod.rs` 에서 모듈 등록

### 3. 데이터베이스 모델 추가

```rust
// core/database/models/new_entity.rs
use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct NewEntity {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewNewEntity {
    pub name: String,
}
```

## 테스트 전략

### 단위 테스트

각 모듈에서 `#[cfg(test)]` 섹션을 사용하여 단위 테스트를 작성합니다:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[tokio::test]
    async fn test_add_product() {
        let service = create_test_inventory_service().await;
        let request = AddProductRequest {
            name: "Test Product".to_string(),
            sku: "TEST001".to_string(),
            quantity: 100,
            price: rust_decimal::Decimal::from(29.99),
        };

        let result = service.add_product(request).await;
        assert!(result.is_ok());
    }
}
```

### 통합 테스트

`tests/integration/` 디렉토리에서 모듈 간 상호작용을 테스트합니다:

```rust
// tests/integration/inventory_tests.rs
use erp_cli::test_utils::TestContext;

#[tokio::test]
async fn test_inventory_crud_operations() {
    let ctx = TestContext::new().await;

    // 제품 추가 테스트
    let product = ctx.inventory_service
        .add_product(/* ... */)
        .await
        .expect("Failed to add product");

    // 제품 조회 테스트
    let found = ctx.inventory_service
        .find_by_sku(&product.sku)
        .await
        .expect("Failed to find product");

    assert_eq!(found.unwrap().id, product.id);
}
```

### 테스트 유틸리티

`tests/common/` 에서 공통 테스트 유틸리티를 제공합니다:

```rust
// tests/common/test_context.rs
pub struct TestContext {
    pub db_pool: Arc<PgPool>,
    pub inventory_service: InventoryService,
    pub customer_service: CustomerService,
    // ...
}

impl TestContext {
    pub async fn new() -> Self {
        let db_pool = create_test_database().await;
        run_migrations(&db_pool).await;

        Self {
            db_pool: db_pool.clone(),
            inventory_service: InventoryService::new(/* ... */),
            // ...
        }
    }
}
```

## 보안 고려사항

### 입력 검증

모든 사용자 입력은 커스텀 검증 유틸리티를 통해 검증됩니다:

```rust
// utils/validation.rs
pub fn validate_email(email: &str) -> ErpResult<()> {
    if email_regex().is_match(email) {
        Ok(())
    } else {
        Err(ErpError::Validation {
            message: "Invalid email format".to_string(),
        })
    }
}
```

### 인증 및 권한

JWT 토큰을 사용한 인증과 RBAC를 통한 권한 관리:

```rust
// core/auth/service.rs
pub struct AuthService {
    jwt_secret: String,
    token_expiry: Duration,
}

impl AuthService {
    pub async fn authenticate(&self, credentials: &Credentials) -> ErpResult<AuthToken> {
        // 인증 로직
    }

    pub fn verify_token(&self, token: &str) -> ErpResult<Claims> {
        // 토큰 검증 로직
    }
}
```

### SQL 인젝션 방지

SQLx prepared statements를 사용하여 SQL 인젝션을 방지합니다:

```rust
// 올바른 방법
let products = sqlx::query_as!(
    Product,
    "SELECT * FROM products WHERE category = $1",
    category
)
.fetch_all(&self.pool)
.await?;
```

## 성능 고려사항

### 연결 풀링

데이터베이스 연결 풀을 사용하여 성능을 최적화합니다:

```rust
// core/database/connection.rs
pub async fn create_pool(database_url: &str) -> ErpResult<PgPool> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await
        .map_err(ErpError::from)
}
```

### 페이지네이션

대용량 데이터 쿼리에 페이지네이션을 구현합니다:

```rust
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
    pub offset: u32,
}

impl Pagination {
    pub fn new(page: u32, limit: u32) -> Self {
        Self {
            page,
            limit,
            offset: (page - 1) * limit,
        }
    }
}
```

### 캐싱

Redis를 사용한 캐싱 (선택사항):

```rust
// utils/cache.rs
pub struct CacheService {
    redis_client: Option<redis::Client>,
}

impl CacheService {
    pub async fn get<T>(&self, key: &str) -> ErpResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        // 캐시 조회 로직
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl: Duration) -> ErpResult<()>
    where
        T: Serialize,
    {
        // 캐시 저장 로직
    }
}
```

## 핵심 의존성

### 주요 크레이트

- **clap**: CLI 인터페이스 (derive 매크로 사용)
- **tokio**: 비동기 런타임
- **sqlx**: 데이터베이스 툴킷 (컴파일 타임 쿼리 검증)
- **serde**: 직렬화/역직렬화
- **tracing**: 구조화된 로깅
- **config**: 설정 관리
- **uuid**: UUID 생성
- **chrono**: 날짜/시간 처리
- **rust_decimal**: 정확한 소수점 연산 (금융 데이터용)
- **bcrypt**: 패스워드 해싱
- **jsonwebtoken**: JWT 토큰 처리
- **thiserror/anyhow**: 에러 처리

### 개발 의존성

- **tokio-test**: 비동기 테스트
- **mockall**: 모킹 프레임워크
- **rstest**: 매개변수화된 테스트
- **criterion**: 벤치마크
- **cargo-tarpaulin**: 코드 커버리지

## CLI 명령어 구조

```bash
erp [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]

# 예시:
erp inventory add "상품명" --quantity 100 --price 29.99
erp customers list --search "고객명"
erp sales create-order --customer ID --product ID --quantity 5
erp reports sales-summary --period monthly
```

각 명령어는 일관된 패턴을 따릅니다:
- 옵션 검증
- 출력 형식 지원 (`tabled`, `comfy-table` 크레이트 사용)
- 에러 처리

## 지속적 통합 및 배포

### GitHub Actions 워크플로 예시

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
    - uses: actions/checkout@v3
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run tests
      run: cargo test

    - name: Run clippy
      run: cargo clippy -- -D warnings

    - name: Check formatting
      run: cargo fmt -- --check
```

## 문제 해결

### 일반적인 개발 이슈

1. **컴파일 오류**
   ```bash
   cargo check
   cargo clippy
   ```

2. **테스트 실패**
   ```bash
   cargo test -- --nocapture
   RUST_LOG=debug cargo test
   ```

3. **성능 문제**
   ```bash
   cargo bench
   RUST_LOG=trace cargo run
   ```

### 데이터베이스 마이그레이션

```bash
# 새로운 마이그레이션 생성
sqlx migrate add create_new_table

# 마이그레이션 실행
sqlx migrate run

# 마이그레이션 되돌리기
sqlx migrate revert
```

## 개발 워크플로

### 브랜치 전략

```bash
# 새로운 기능 개발
git checkout -b feature/inventory-enhancement
git push -u origin feature/inventory-enhancement

# 버그 수정
git checkout -b bugfix/authentication-issue
git push -u origin bugfix/authentication-issue

# 핫픽스
git checkout -b hotfix/critical-security-fix
git push -u origin hotfix/critical-security-fix
```

### 코드 리뷰 프로세스

1. **Pre-commit 체크리스트**
   ```bash
   # 코드 포맷팅
   cargo fmt --check

   # 린트 체크
   cargo clippy -- -D warnings

   # 테스트 실행
   cargo test

   # 문서 빌드
   cargo doc --no-deps
   ```

2. **Pull Request 생성**
   - 명확한 제목과 설명
   - 변경사항 요약
   - 테스트 계획 포함
   - 스크린샷 또는 로그 첨부 (필요시)

3. **리뷰 기준**
   - 코드 품질 및 가독성
   - 테스트 커버리지
   - 성능 영향 분석
   - 보안 고려사항
   - 문서화 완성도

### 릴리스 프로세스

```bash
# 버전 태그 생성
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0

# 릴리스 바이너리 빌드
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin
```

## 디버깅 및 프로파일링

### 로그 레벨별 디버깅

```bash
# 상세 디버그 정보
RUST_LOG=trace cargo run -- inventory list

# 특정 모듈만 디버그
RUST_LOG=erp_cli::modules::inventory=debug cargo run

# JSON 형식 로그
RUST_LOG=info RUST_LOG_FORMAT=json cargo run
```

### 성능 프로파일링

```bash
# 벤치마크 실행
cargo bench

# CPU 프로파일링 (perf 필요)
perf record --call-graph=dwarf ./target/release/erp inventory list
perf report

# 메모리 프로파일링 (valgrind 필요)
valgrind --tool=massif ./target/release/erp
```

### 데이터베이스 디버깅

```bash
# SQL 쿼리 로깅 활성화
SQLX_LOGGING=true cargo run

# 데이터베이스 연결 디버그
DATABASE_URL="postgresql://..." RUST_LOG=sqlx=debug cargo run
```

## 지속적 통합 및 배포 (CI/CD)

### GitHub Actions 워크플로

```yaml
# .github/workflows/ci.yml (완전한 예시)
name: Continuous Integration

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: erp_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy

    - name: Setup Rust cache
      uses: Swatinem/rust-cache@v2

    - name: Run tests
      env:
        DATABASE_URL: postgres://postgres:postgres@localhost/erp_test
      run: cargo test --verbose

    - name: Check formatting
      run: cargo fmt -- --check

    - name: Run clippy
      run: cargo clippy -- -D warnings

    - name: Generate documentation
      run: cargo doc --no-deps

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Install cargo-audit
      run: cargo install cargo-audit
    - name: Run security audit
      run: cargo audit

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin
    - name: Generate coverage
      run: cargo tarpaulin --out Xml
    - name: Upload to codecov
      uses: codecov/codecov-action@v3
```

### 배포 자동화

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build and Release
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: erp
            asset_name: erp-linux-amd64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: erp.exe
            asset_name: erp-windows-amd64.exe
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: erp
            asset_name: erp-macos-amd64

    runs-on: ${{ matrix.os }}

    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: ${{ matrix.target }}

    - name: Build release
      run: cargo build --release --target ${{ matrix.target }}

    - name: Upload release asset
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ steps.create_release.outputs.upload_url }}
        asset_path: ./target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
        asset_name: ${{ matrix.asset_name }}
        asset_content_type: application/octet-stream
```

## 패키징 및 배포

### Docker 컨테이너화

```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/erp /usr/local/bin/erp

EXPOSE 8080
CMD ["erp", "server", "start"]
```

```yaml
# docker-compose.yml
version: '3.8'

services:
  erp:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/erp
      - RUST_LOG=info
    depends_on:
      - db
      - redis

  db:
    image: postgres:13
    environment:
      - POSTGRES_DB=erp
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

volumes:
  postgres_data:
```

### 시스템 패키지

```bash
# Debian/Ubuntu 패키지 생성 (cargo-deb 필요)
cargo install cargo-deb
cargo deb

# RPM 패키지 생성 (cargo-rpm 필요)
cargo install cargo-rpm
cargo rpm build
```

### 설치 스크립트

```bash
#!/bin/bash
# install.sh
set -e

# 플랫폼 감지
PLATFORM=$(uname -s)
ARCH=$(uname -m)

case $PLATFORM in
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_URL="https://github.com/example/erp-cli/releases/latest/download/erp-linux-amd64"
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    Darwin)
        BINARY_URL="https://github.com/example/erp-cli/releases/latest/download/erp-macos-amd64"
        ;;
    *)
        echo "Unsupported platform: $PLATFORM"
        exit 1
        ;;
esac

# 다운로드 및 설치
echo "Downloading ERP CLI..."
curl -L "$BINARY_URL" -o /tmp/erp
chmod +x /tmp/erp
sudo mv /tmp/erp /usr/local/bin/erp

echo "ERP CLI installed successfully!"
echo "Run 'erp --help' to get started."
```

## 기여 가이드라인

### 코드 스타일 가이드

1. **Rust 표준 스타일 준수**
   ```bash
   # 포맷팅 적용
   cargo fmt

   # 포맷팅 체크
   cargo fmt -- --check
   ```

2. **Clippy 경고 해결**
   ```bash
   # 모든 경고를 에러로 처리
   cargo clippy -- -D warnings

   # 특정 린트 허용 (필요시)
   #[allow(clippy::similar_names)]
   ```

3. **문서화 표준**
   ```rust
   /// 제품을 재고에 추가합니다.
   ///
   /// # Arguments
   ///
   /// * `product` - 추가할 제품 정보
   ///
   /// # Returns
   ///
   /// 성공 시 생성된 제품의 ID를 반환합니다.
   ///
   /// # Errors
   ///
   /// 다음의 경우 에러를 반환합니다:
   /// * SKU가 이미 존재하는 경우
   /// * 데이터베이스 연결 실패
   ///
   /// # Examples
   ///
   /// ```rust
   /// let product = NewProduct {
   ///     name: "Test Product".to_string(),
   ///     sku: "TEST001".to_string(),
   ///     quantity: 100,
   ///     price: Decimal::from(29.99),
   /// };
   ///
   /// let result = service.add_product(product).await?;
   /// ```
   pub async fn add_product(&self, product: NewProduct) -> ErpResult<Uuid> {
       // 구현...
   }
   ```

### 테스트 작성 가이드

1. **단위 테스트**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use crate::test_utils::*;

       #[tokio::test]
       async fn test_add_product_success() {
           // Arrange
           let service = create_test_inventory_service().await;
           let product = create_test_product();

           // Act
           let result = service.add_product(product).await;

           // Assert
           assert!(result.is_ok());
           let product_id = result.unwrap();
           assert!(!product_id.is_nil());
       }

       #[tokio::test]
       async fn test_add_product_duplicate_sku() {
           // 중복 SKU 테스트
       }
   }
   ```

2. **통합 테스트**
   ```rust
   // tests/integration/inventory_tests.rs
   use erp_cli::test_utils::TestContext;

   #[tokio::test]
   async fn test_full_inventory_workflow() {
       let ctx = TestContext::new().await;

       // 제품 추가
       let product = ctx.add_test_product().await;

       // 재고 조회
       let inventory = ctx.inventory_service.list_products().await.unwrap();
       assert!(!inventory.is_empty());

       // 제품 업데이트
       ctx.inventory_service.update_product(
           &product.sku,
           UpdateProductRequest { quantity: Some(200), ..Default::default() }
       ).await.unwrap();

       // 제품 삭제
       ctx.inventory_service.remove_product(&product.sku).await.unwrap();
   }
   ```

### 커밋 메시지 컨벤션

```bash
# 형식: <타입>(<범위>): <제목>
#
# 타입:
# - feat: 새로운 기능
# - fix: 버그 수정
# - docs: 문서 변경
# - style: 코드 스타일 변경 (포맷팅, 세미콜론 등)
# - refactor: 리팩토링
# - test: 테스트 추가 또는 수정
# - chore: 빌드 프로세스 또는 보조 도구 변경

# 예시:
feat(inventory): add product search functionality
fix(auth): resolve JWT token validation issue
docs(api): update API documentation for sales module
refactor(database): optimize database connection pooling
test(customers): add integration tests for customer service
```

### Pull Request 템플릿

```markdown
## 변경사항 요약
<!-- 이 PR에서 무엇을 변경했는지 간략하게 설명하세요 -->

## 변경 타입
- [ ] 🚀 새로운 기능 (feat)
- [ ] 🐛 버그 수정 (fix)
- [ ] 📚 문서 업데이트 (docs)
- [ ] 🎨 코드 스타일 변경 (style)
- [ ] ♻️ 리팩토링 (refactor)
- [ ] ✅ 테스트 추가/수정 (test)
- [ ] 🔧 기타 변경사항 (chore)

## 테스트
- [ ] 기존 테스트가 통과합니다
- [ ] 새로운 테스트를 추가했습니다
- [ ] 수동 테스트를 완료했습니다

## 체크리스트
- [ ] 코드가 프로젝트의 스타일 가이드를 따릅니다
- [ ] 자체 리뷰를 완료했습니다
- [ ] 코드에 명확한 주석을 추가했습니다
- [ ] 문서를 업데이트했습니다
- [ ] 변경사항이 기존 기능을 깨뜨리지 않습니다

## 관련 이슈
<!-- 관련된 이슈 번호를 적어주세요 (예: Closes #123) -->

## 스크린샷 (필요시)
<!-- 화면 변경사항이 있는 경우 스크린샷을 첨부하세요 -->
```

## 리소스

### 공식 문서
- [Rust 공식 문서](https://doc.rust-lang.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

### 주요 크레이트 문서
- [Tokio 가이드](https://tokio.rs/tokio/tutorial)
- [SQLx 문서](https://docs.rs/sqlx/)
- [Clap 가이드](https://docs.rs/clap/)
- [Tracing 가이드](https://docs.rs/tracing/)
- [Serde 가이드](https://serde.rs/)

### 개발 도구
- [Rust Analyzer](https://rust-analyzer.github.io/) - IDE 지원
- [cargo-edit](https://github.com/killercup/cargo-edit) - 의존성 관리
- [cargo-watch](https://github.com/watchexec/cargo-watch) - 파일 변경 감지
- [cargo-expand](https://github.com/dtolnay/cargo-expand) - 매크로 확장 보기

### 성능 및 디버깅
- [flamegraph](https://github.com/flamegraph-rs/flamegraph) - 성능 프로파일링
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat) - 바이너리 크기 분석
- [tokio-console](https://github.com/tokio-rs/console) - Tokio 런타임 모니터링