# ERP CLI 시스템

Rust로 구축된 고성능 모듈형 전사적 자원 관리(ERP) CLI 시스템입니다. 중소기업의 재고, 판매, 고객 관리 및 종합 보고서 생성을 위해 설계되었습니다.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## 주요 기능

- **재고 관리** - 제품, 재고 수준, 가격, 카테고리 추적 ✅ 검증완료
- **판매 관리** - 주문 처리, 인보이스 관리, 판매 성과 추적 ✅ 검증완료
- **고객 관리** - 고객 정보 및 관계 이력 유지 ✅ 검증완료
- **보고서 시스템** - 종합적인 비즈니스 보고서 및 분석 생성 ✅ 검증완료
- **설정 관리** - 유연한 시스템 설정 및 사용자 선호도 ✅ 검증완료
- **데이터베이스 마이그레이션** - 자동화된 데이터베이스 스키마 관리 ✅ 검증완료
- **인증 및 권한 부여** - 역할 기반 액세스 제어가 포함된 JWT 인증
- **크로스 플랫폼** - Windows, Linux, macOS에서 실행 가능

## 아키텍처

시스템은 4계층 모듈형 아키텍처를 따릅니다:

1. **CLI 인터페이스 계층** - 명령 파싱, 검증, 사용자 상호작용
2. **비즈니스 로직 계층** - 핵심 비즈니스 모듈 (재고, 판매, 고객, 보고서, 설정)
3. **핵심 서비스 계층** - 인증, 데이터베이스, 설정, 로깅 서비스
4. **데이터 계층** - PostgreSQL (운영), SQLite (개발), Redis (캐싱)

## 빠른 시작

### 필수 요구사항

- Rust 1.70+ ([rustup.rs](https://rustup.rs/)에서 설치)
- Git
- 선택사항: 운영 환경용 PostgreSQL

### 설치

1. 저장소를 복제합니다:
   ```bash
   git clone https://github.com/example/erp-cli.git
   cd erp-cli
   ```

2. 프로젝트를 빌드합니다:
   ```bash
   cargo build --release
   ```

3. 데이터베이스를 초기화합니다:
   ```bash
   ./target/release/erp setup --init-db
   ```

4. 첫 번째 명령을 실행합니다:
   ```bash
   ./target/release/erp --help
   ```

## 사용법

### 기본 명령어

```bash
# 재고 관리
erp inventory add "제품명" --quantity 100 --price 29.99 --category electronics
erp inventory list --category electronics
erp inventory update PRODUCT_ID --quantity 150

# 고객 관리
erp customers add "홍길동" --email hong@example.com --phone "+821234567890"
erp customers list --search "홍길동"
erp customers update CUSTOMER_ID --email newemail@example.com

# 판매 관리
erp sales create-order --customer CUSTOMER_ID --product PRODUCT_ID --quantity 5
erp sales list-orders --status pending
erp sales invoice ORDER_ID

# 보고서
erp reports sales-summary --period monthly
erp reports inventory-status
erp reports customer-analytics

# 설정
erp config set database.url "postgresql://user:pass@localhost/erp"
erp config get database.url
erp config list

# 데이터베이스 관리
erp migrate run
erp migrate status
```

### 설정

시스템은 TOML 설정 파일을 사용합니다:

- **개발**: `config/development.toml`
- **운영**: `config/production.toml`
- **환경 변수**: `ERP_*` 접두사 변수로 모든 설정 재정의 가능

설정 예시:
```toml
[database]
url = "sqlite://erp.db"
max_connections = 10
min_connections = 1

[server]
host = "0.0.0.0"
port = 8080

[auth]
jwt_secret = "your-secret-key"
jwt_expires_in = 3600

[logging]
level = "info"
format = "json"
```

## 개발

### 소스에서 빌드하기

```bash
# 개발 빌드
cargo build

# 최적화된 릴리스 빌드
cargo build --release

# cargo로 실행
cargo run -- --help
```

### 테스트

```bash
# 모든 테스트 실행
cargo test

# 테스트 커버리지와 함께 실행
cargo tarpaulin --out Html

# 특정 모듈 테스트 실행
cargo test inventory
```

### 코드 품질

```bash
# 코드 포맷팅
cargo fmt

# 린터 실행
cargo clippy -- -D warnings

# 빌드 없이 컴파일 확인
cargo check
```

### 프로젝트 구조

```
src/
├── cli/                 # CLI 인터페이스 계층
│   ├── commands/        # 명령어 정의
│   ├── parser.rs        # 명령어 파싱
│   └── validator.rs     # 입력 검증
├── modules/             # 비즈니스 로직 계층
│   ├── inventory/       # 재고 관리
│   ├── sales/           # 판매 처리
│   ├── customers/       # 고객 관리
│   ├── reports/         # 보고서 생성
│   └── config/          # 설정 관리
├── core/                # 핵심 서비스 계층
│   ├── auth/           # 인증 및 권한 부여
│   ├── database/       # 데이터베이스 작업
│   ├── config/         # 설정 로딩
│   └── logging/        # 로깅 서비스
├── utils/              # 공유 유틸리티
│   ├── error.rs        # 에러 처리
│   ├── crypto.rs       # 암호화 유틸리티
│   └── validation.rs   # 입력 검증
└── main.rs             # 애플리케이션 진입점
```

## 데이터베이스 스키마

시스템은 SQLite (개발) 및 PostgreSQL (운영) 모두 지원합니다:

### 핵심 테이블
- **users** - 사용자 인증 및 프로필
- **products** - SKU, 가격, 카테고리가 포함된 재고 항목
- **customers** - 고객 정보 및 연락처 세부사항
- **orders** - 상태 추적이 포함된 판매 주문
- **order_items** - 주문과 제품을 연결하는 라인 항목

### 주요 특징
- **마이그레이션** - 자동화된 스키마 버전 관리
- **연결 풀링** - 효율적인 데이터베이스 연결
- **쿼리 최적화** - 준비된 문과 인덱싱
- **데이터 검증** - Rust 타입 시스템을 활용한 강력한 타이핑

## 보안

- **비밀번호 해싱** - 설정 가능한 라운드를 가진 bcrypt
- **JWT 인증** - 상태 비저장 토큰 기반 인증
- **역할 기반 액세스 제어** - 세분화된 권한
- **SQL 인젝션 방지** - SQLx를 사용한 매개변수화된 쿼리
- **입력 검증** - 종합적인 데이터 정제
- **환경 변수** - 보안 설정 관리

## 성능

- **Async/Await** - Tokio를 사용한 비차단 I/O 작업
- **연결 풀링** - 효율적인 데이터베이스 리소스 관리
- **페이지네이션** - 메모리 효율적인 대용량 데이터 처리
- **캐싱** - 자주 액세스되는 데이터를 위한 Redis 지원
- **최적화된 빌드** - 운영용 LTO 및 최적화 플래그

## 주요 의존성

### 핵심 의존성
- **clap** - 명령줄 인수 파싱
- **tokio** - 비동기 런타임
- **sqlx** - 컴파일 타임 검증을 지원하는 데이터베이스 툴킷
- **serde** - 직렬화/역직렬화
- **tracing** - 구조화된 로깅
- **config** - 설정 관리

### 비즈니스 로직
- **uuid** - 고유 식별자 생성
- **chrono** - 날짜 및 시간 처리
- **rust_decimal** - 금융 데이터용 정밀 소수점 연산
- **bcrypt** - 비밀번호 해싱
- **jsonwebtoken** - JWT 토큰 처리

### 개발 도구
- **cargo-nextest** - 빠른 테스트 러너
- **cargo-tarpaulin** - 코드 커버리지 분석
- **mockall** - 테스트용 모의 객체 생성
- **rstest** - 매개변수화된 테스팅

## 기여하기

1. 저장소를 포크합니다
2. 기능 브랜치를 생성합니다 (`git checkout -b feature/amazing-feature`)
3. 코딩 표준에 따라 변경사항을 만듭니다
4. 새로운 기능에 대한 테스트를 추가합니다
5. 테스트 스위트를 실행합니다 (`cargo test`)
6. 변경사항을 커밋합니다 (`git commit -m 'Add amazing feature'`)
7. 브랜치에 푸시합니다 (`git push origin feature/amazing-feature`)
8. Pull Request를 엽니다

### 코딩 표준

- Rust의 공식 스타일 가이드라인을 따릅니다
- 일관된 포맷팅을 위해 `cargo fmt`를 사용합니다
- 모든 `cargo clippy` 경고가 해결되도록 합니다
- 새로운 기능에 대한 종합적인 테스트를 작성합니다
- rustdoc 주석으로 공개 API를 문서화합니다
- 기존의 에러 처리 패턴을 따릅니다

## 라이선스

이 프로젝트는 MIT 라이선스 하에 라이선스됩니다 - 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

## 지원

- **이슈** - GitHub Issues를 통해 버그를 신고하고 기능을 요청하세요
- **토론** - 커뮤니티 토론에 참여하세요
- **문서** - `/docs` 디렉터리에서 종합적인 문서를 확인할 수 있습니다

## 로드맵

- [ ] 웹 대시보드 인터페이스
- [ ] RESTful API 서버
- [ ] 멀티 테넌트 지원
- [ ] 차트를 포함한 고급 보고서
- [ ] 외부 회계 시스템과의 통합
- [ ] 모바일 컴패니언 앱
- [ ] Docker 컨테이너화
- [ ] Kubernetes 배포 매니페스트

---

Rust로 ❤️를 담아 제작했습니다