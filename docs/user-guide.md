# ERP CLI 사용자 가이드

## 개요

ERP CLI는 중소기업을 위한 포괄적인 명령줄 기반 ERP(Enterprise Resource Planning) 시스템입니다. 재고 관리, 고객 관리, 영업 관리, 보고서 생성 등의 핵심 비즈니스 기능을 제공합니다.

## 설치

### 요구사항

- Rust 1.70 이상
- PostgreSQL 13 이상 (프로덕션) 또는 SQLite (개발)
- Redis (선택사항, 캐싱용)

### 빌드

```bash
# 소스 코드 클론
git clone <repository-url>
cd erp

# 개발 빌드
cargo build

# 릴리스 빌드
cargo build --release
```

## 초기 설정

### 1. 데이터베이스 초기화

```bash
# 개발 환경 (SQLite)
./target/release/erp setup --init-db

# 프로덕션 환경 (PostgreSQL)
export DATABASE_URL="postgresql://username:password@localhost/erp_db"
./target/release/erp setup --init-db
```

### 2. 관리자 계정 생성

```bash
erp config set admin_user "admin@company.com"
erp config set admin_password "secure_password"
```

## 핵심 기능

### 재고 관리 (Inventory)

#### 제품 추가

```bash
# 기본 제품 추가
erp inventory add "MacBook Pro" --sku "MBP001" --quantity 10 --price 1999.99

# 카테고리와 설명 포함
erp inventory add "iPhone 15" \
  --sku "IP15001" \
  --category "전자제품" \
  --quantity 50 \
  --price 799.99 \
  --description "최신 iPhone 모델"
```

#### 재고 목록 조회

```bash
# 모든 재고 조회
erp inventory list

# 카테고리별 조회
erp inventory list --category "전자제품"

# 제품 검색
erp inventory list --search "iPhone"

# JSON 형식으로 출력
erp inventory list --format json

# 가격순 정렬
erp inventory list --sort-by price --order desc

# 낮은 재고 알림
erp inventory low-stock --threshold 10
```

#### 재고 업데이트

```bash
# 가격 수정
erp inventory update MBP001 --price 1899.99

# 재고 수량 수정
erp inventory update MBP001 --quantity 25

# 여러 필드 동시 수정
erp inventory update MBP001 --price 1799.99 --quantity 15 --description "할인 상품"
```

#### 재고 삭제

```bash
# 제품 삭제 (확인 프롬프트 표시)
erp inventory remove MBP001

# 강제 삭제
erp inventory remove MBP001 --force
```

### 고객 관리 (Customers)

#### 고객 추가

```bash
# 개인 고객 추가
erp customers add "김철수" \
  --email "kim@example.com" \
  --phone "010-1234-5678" \
  --address "서울시 강남구"

# 기업 고객 추가
erp customers add "ABC 회사" \
  --email "contact@abc.com" \
  --phone "02-1234-5678" \
  --company "ABC Corporation" \
  --notes "주요 거래처"
```

#### 고객 목록 조회

```bash
# 모든 고객 조회
erp customers list

# 페이지네이션과 정렬
erp customers list --page 1 --limit 20 --sort-by name --order asc

# JSON 형식으로 출력
erp customers list --format json
```

#### 고객 검색

```bash
# 고객 검색
erp customers search --query "김철수"

# JSON 형식으로 검색 결과
erp customers search --query "ABC" --format json
```

#### 고객 정보 업데이트

```bash
# 이메일 변경
erp customers update CUSTOMER_ID --email "new@example.com"

# 여러 필드 동시 수정
erp customers update CUSTOMER_ID \
  --name "김철수" \
  --phone "010-9876-5432" \
  --address "부산시 해운대구"
```

#### 고객 삭제

```bash
# 고객 삭제 (확인 프롬프트 표시)
erp customers delete CUSTOMER_ID

# 강제 삭제
erp customers delete CUSTOMER_ID --force
```

### 영업 관리 (Sales)

#### 주문 생성

```bash
# 기본 주문 생성
erp sales create-order \
  --customer-id "550e8400-e29b-41d4-a716-446655440000" \
  --product-sku "MBP001" \
  --quantity 2

# 메모 포함 주문
erp sales create-order \
  --customer-id "550e8400-e29b-41d4-a716-446655440000" \
  --product-sku "IP15001" \
  --quantity 1 \
  --notes "긴급 주문"
```

#### 주문 목록 조회

```bash
# 모든 주문 조회
erp sales list-orders

# 특정 기간 주문 조회
erp sales list-orders --from "2024-01-01" --to "2024-01-31"

# 상태별 주문 조회
erp sales list-orders --status "pending"

# 특정 고객의 주문
erp sales list-orders --customer-id "550e8400-e29b-41d4-a716-446655440000"

# JSON 형식으로 출력
erp sales list-orders --format json
```

#### 주문 상태 업데이트

```bash
# 주문 상태 변경
erp sales update-order "ORDER_ID" --status "shipped"

# 메모와 함께 상태 변경
erp sales update-order "ORDER_ID" \
  --status "delivered" \
  --notes "고객이 수령 확인함"
```

#### 인보이스 생성

```bash
# 기본 인보이스 생성
erp sales generate-invoice "ORDER_ID"

# 특정 경로에 저장
erp sales generate-invoice "ORDER_ID" \
  --output "/path/to/invoice.pdf"

# JSON 형식으로 생성
erp sales generate-invoice "ORDER_ID" --format json
```

### 보고서 (Reports)

#### 매출 요약 보고서

```bash
# 월별 매출 요약
erp reports sales-summary --period monthly

# 특정 기간 매출 요약
erp reports sales-summary --from "2024-01-01" --to "2024-01-31"

# CSV 파일로 저장
erp reports sales-summary --period monthly --format csv --output "sales_2024_01.csv"

# PDF 보고서 생성
erp reports sales-summary --period yearly --format pdf --output "annual_sales.pdf"
```

#### 재고 상태 보고서

```bash
# 전체 재고 상태
erp reports inventory-status

# 저재고 상품만
erp reports inventory-status --low-stock-only --threshold 5

# 특정 카테고리의 재고 상태
erp reports inventory-status --category "전자제품"

# JSON 형식으로 저장
erp reports inventory-status --format json --output "inventory_status.json"
```

#### 고객 분석 보고서

```bash
# 상위 10명 매출 고객
erp reports customer-analysis --top 10 --metric revenue

# 주문 빈도 기준 분석
erp reports customer-analysis --metric frequency --period quarterly

# 분기별 고객 분석
erp reports customer-analysis --top 20 --period quarterly --format csv
```

#### 재무 개요 보고서

```bash
# 기본 재무 개요
erp reports financial-overview

# PDF로 내보내기
erp reports financial-overview --export pdf --output "financial_overview.pdf"

# CSV로 내보내기
erp reports financial-overview --export csv --output "financial_data.csv"

# 차트 포함 월별 보고서
erp reports financial-overview --period monthly --include-charts
```

### 설정 관리 (Config)

#### 설정 조회

```bash
# 모든 설정 조회
erp config list

# JSON 형식으로 설정 조회
erp config list --format json

# 특정 패턴의 설정만 조회
erp config list --filter "database.*"

# 특정 설정 조회
erp config get database.url
erp config get auth.jwt_secret
```

#### 설정 변경

```bash
# 통화 설정
erp config set currency "KRW"

# 타임존 설정
erp config set timezone "Asia/Seoul"

# 로그 레벨 설정
erp config set logging.level "debug"

# 데이터베이스 URL 설정
erp config set database.url "postgresql://user:pass@localhost/erp"
```

#### 설정 초기화

```bash
# 설정 초기화 (확인 프롬프트 표시)
erp config reset

# 강제 초기화
erp config reset --confirm
```

#### 설정 파일 경로 확인

```bash
# 현재 사용 중인 설정 파일 경로 표시
erp config path
```

### 데이터베이스 마이그레이션 (Migrate)

#### 데이터베이스 초기화

```bash
# 처음 실행 시 데이터베이스 초기화
erp migrate init

# 기존 데이터베이스 덮어쓰기
erp migrate init --force
```

#### 마이그레이션 실행

```bash
# 최신 상태로 마이그레이션 실행
erp migrate up

# 특정 단계 수만큼 마이그레이션
erp migrate up --steps 3
```

#### 마이그레이션 롤백

```bash
# 1단계 롤백
erp migrate down

# 여러 단계 롤백
erp migrate down --steps 2
```

#### 마이그레이션 상태 확인

```bash
# 현재 마이그레이션 상태 확인
erp migrate status
```

#### 새 마이그레이션 생성

```bash
# 새 테이블 마이그레이션 생성
erp migrate generate create_products_table

# 컬럼 추가 마이그레이션 생성
erp migrate generate add_description_to_products
```

#### 데이터베이스 연결 테스트

```bash
# 데이터베이스 연결 테스트
erp migrate test
```

## 고급 기능

### 데이터 백업

```bash
# 데이터베이스 백업
erp backup create --output "backup_$(date +%Y%m%d).sql"

# 백업 복원
erp backup restore --input "backup_20240115.sql"
```

### 사용자 관리

```bash
# 사용자 추가
erp users add --username "employee1" --role "manager" --email "emp1@company.com"

# 사용자 역할 변경
erp users update --username "employee1" --role "admin"
```

### 감사 로그 조회

```bash
# 최근 활동 조회
erp audit list --limit 50

# 특정 사용자 활동 조회
erp audit list --user "admin" --action "login"
```

## 출력 형식

대부분의 명령어는 다양한 출력 형식을 지원합니다:

- `--format table` (기본값): 표 형식
- `--format json`: JSON 형식
- `--format csv`: CSV 형식
- `--format yaml`: YAML 형식

예시:
```bash
erp inventory list --format json
erp customers list --format csv > customers.csv
```

## 환경 변수

주요 환경 변수들:

- `DATABASE_URL`: 데이터베이스 연결 문자열
- `REDIS_URL`: Redis 연결 문자열 (선택사항)
- `JWT_SECRET`: JWT 토큰 비밀키
- `LOG_LEVEL`: 로그 레벨 (debug, info, warn, error)
- `ERP_CONFIG_PATH`: 설정 파일 경로

## 문제 해결

### 일반적인 오류

1. **데이터베이스 연결 오류**
   ```bash
   # 데이터베이스 상태 확인
   erp status

   # 연결 테스트
   erp config get database_url
   ```

2. **권한 오류**
   ```bash
   # 현재 사용자 확인
   erp whoami

   # 권한 확인
   erp users permissions
   ```

3. **설정 오류**
   ```bash
   # 설정 검증
   erp config validate

   # 기본 설정 복원
   erp config reset
   ```

### 로그 확인

로그는 다음 위치에 저장됩니다:
- Linux/macOS: `~/.local/share/erp/logs/`
- Windows: `%APPDATA%\erp\logs\`

```bash
# 실시간 로그 모니터링
tail -f ~/.local/share/erp/logs/erp.log
```

## 성능 최적화

### 대용량 데이터 처리

```bash
# 페이지네이션 사용
erp inventory list --page 1 --limit 100

# 인덱스 최적화
erp maintenance optimize-indexes

# 통계 업데이트
erp maintenance update-stats
```

### 캐시 관리

```bash
# 캐시 상태 확인
erp cache status

# 캐시 클리어
erp cache clear

# 캐시 워밍업
erp cache warmup
```

## 통합 및 자동화

### 스크립트 예시

```bash
#!/bin/bash
# 일일 보고서 자동 생성 스크립트

DATE=$(date +%Y-%m-%d)
REPORT_DIR="/reports/$DATE"

mkdir -p "$REPORT_DIR"

# 매출 보고서
erp reports sales-summary --period daily --format csv > "$REPORT_DIR/sales.csv"

# 재고 보고서
erp reports inventory-status --format json > "$REPORT_DIR/inventory.json"

# 재무 보고서
erp reports financial-overview --format pdf > "$REPORT_DIR/financial.pdf"

echo "일일 보고서가 $REPORT_DIR 에 생성되었습니다."
```

### API 통합

ERP CLI는 REST API도 제공합니다 (서버 모드):

```bash
# API 서버 시작
erp server start --port 8080

# API 문서 확인
curl http://localhost:8080/api/docs
```

## 보안 권장사항

1. **강력한 패스워드 사용**
2. **정기적인 백업**
3. **접근 권한 최소화**
4. **감사 로그 모니터링**
5. **SSL/TLS 사용 (프로덕션)**

## 지원 및 문의

- 문서: [프로젝트 Wiki](링크)
- 이슈 트래킹: [GitHub Issues](링크)
- 이메일: support@company.com

## 라이선스

이 소프트웨어는 MIT 라이선스 하에 배포됩니다.