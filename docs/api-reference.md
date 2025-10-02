# ERP CLI API 레퍼런스

이 문서는 ERP CLI 시스템의 모든 명령어와 옵션에 대한 종합적인 레퍼런스입니다.

## 📌 개발 환경 사용 안내

개발 환경에서는 컴파일된 바이너리 대신 `cargo run --`을 사용합니다:

```bash
# 개발 환경
cargo run -- inventory list

# 배포된 바이너리
erp inventory list
```

문서의 모든 예시는 두 가지 형태로 제공됩니다.

## 목차

- [전역 옵션](#전역-옵션)
- [재고 관리 (inventory)](#재고-관리-inventory)
- [고객 관리 (customers)](#고객-관리-customers)
- [영업 관리 (sales)](#영업-관리-sales)
- [보고서 (reports)](#보고서-reports)
- [설정 관리 (config)](#설정-관리-config)
- [마이그레이션 (migrate)](#마이그레이션-migrate)
- [응답 형식](#응답-형식)
- [에러 코드](#에러-코드)

## 전역 옵션

모든 명령어에서 사용할 수 있는 전역 옵션들입니다.

### 사용법

#### 배포된 바이너리 사용
```
erp [전역옵션] <명령어> [하위명령어] [옵션] [인수]
```

#### 개발 환경에서 실행
```
cargo run -- [전역옵션] <명령어> [하위명령어] [옵션] [인수]
```

### 전역 옵션

| 옵션 | 짧은 형태 | 설명 | 기본값 |
|------|-----------|------|--------|
| `--config <CONFIG>` | | 설정 파일 경로 지정 | 환경에 따라 자동 선택 |
| `--log-level <LOG_LEVEL>` | | 로그 레벨 설정 | `info` |
| `--help` | `-h` | 도움말 표시 | |
| `--version` | `-V` | 버전 정보 표시 | |

### 로그 레벨 옵션
- `trace`: 모든 상세 로그
- `debug`: 디버그 정보 포함
- `info`: 일반 정보
- `warn`: 경고 메시지만
- `error`: 에러 메시지만

### 예시

#### 배포된 바이너리 사용
```bash
# 디버그 모드로 실행
erp --log-level debug inventory list

# 커스텀 설정 파일 사용
erp --config /path/to/config.toml customers list

# 버전 확인
erp --version
```

#### 개발 환경에서 실행
```bash
# 디버그 모드로 실행
cargo run -- --log-level debug inventory list

# 커스텀 설정 파일 사용
cargo run -- --config /path/to/config.toml customers list

# 버전 확인
cargo run -- --version
```

---

## 재고 관리 (inventory)

제품 재고를 관리하는 명령어들입니다.

### inventory add - 제품 추가

새로운 제품을 재고에 추가합니다.

#### 사용법
```bash
erp inventory add <제품명> [옵션]
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<제품명>` | 추가할 제품의 이름 |

#### 옵션
| 옵션 | 설명 | 필수 | 기본값 | 제약 조건 |
|------|------|------|-------|----------|
| `--sku <SKU>` | 제품 식별 코드 | ✓ | | 영문, 숫자, -, _ 만 허용 |
| `--category <카테고리>` | 제품 카테고리 | | general | |
| `--quantity <수량>` | 초기 재고 수량 | ✓ | | 1 ~ 2,147,483,647 |
| `--price <가격>` | 제품 가격 | ✓ | | 0.01 ~ 99,999,999,999,999.99 |
| `--cost <원가>` | 제품 원가 | | 가격의 70% | 0.01 ~ 99,999,999,999,999.99 |
| `--description <설명>` | 제품 설명 | | | |

#### 검증 규칙

- **제품명**: 비어있을 수 없으며, 최대 255자
- **SKU**: 비어있을 수 없으며, 영문/숫자/하이픈/언더스코어만 허용, 최대 50자
- **수량**: 1 이상, 최대 2,147,483,647 (i32 최대값)
- **가격**: 0.01 이상, 최대 99,999,999,999,999.99
- **원가**: 0.01 이상, 최대 99,999,999,999,999.99 (미지정 시 가격의 70%로 자동 계산)
- **카테고리**: 미지정 시 "general"로 설정

**참고**: 원가는 0 또는 음수가 될 수 없습니다. 무료 증정품이라도 실제 원가는 존재해야 합니다.

#### 예시

##### 배포된 바이너리 사용
```bash
# 기본 제품 추가 (원가 자동 계산: 가격의 70%)
erp inventory add "MacBook Pro" --sku "MBP001" --quantity 10 --price 1999.99

# 원가 직접 지정
erp inventory add "iPhone 15" \
  --sku "IP15001" \
  --category "전자제품" \
  --quantity 50 \
  --price 799.99 \
  --cost 500.00 \
  --description "최신 iPhone 모델"

# 카테고리와 설명 포함 (원가는 기본값 사용)
erp inventory add "iPad Pro" \
  --sku "IPAD001" \
  --category "전자제품" \
  --quantity 25 \
  --price 1299.99 \
  --description "프로용 태블릿"
```

##### 개발 환경에서 실행
```bash
# 기본 제품 추가 (원가 자동 계산: 가격의 70%)
cargo run -- inventory add "MacBook Pro" --sku "MBP001" --quantity 10 --price 1999.99

# 원가 직접 지정
cargo run -- inventory add "iPhone 15" \
  --sku "IP15001" \
  --category "전자제품" \
  --quantity 50 \
  --price 799.99 \
  --cost 500.00 \
  --description "최신 iPhone 모델"

# 카테고리와 설명 포함 (원가는 기본값 사용)
cargo run -- inventory add "iPad Pro" \
  --sku "IPAD001" \
  --category "전자제품" \
  --quantity 25 \
  --price 1299.99 \
  --description "프로용 태블릿"
```

#### 응답
```json
{
  "status": "success",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "MacBook Pro",
    "sku": "MBP001",
    "category": "전자제품",
    "quantity": 10,
    "price": 1999.99,
    "cost": 1399.99,
    "margin": 600.00,
    "margin_percentage": 42.86,
    "description": null,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

### inventory list - 제품 목록 조회

재고 목록을 조회합니다. 각 제품의 SKU, 제품명, 카테고리, 가격, **원가**, 수량, 상태, 마진율이 표시됩니다.

#### 사용법
```bash
erp inventory list [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--category <카테고리>` | 특정 카테고리 필터링 | 모든 카테고리 |
| `--search <검색어>` | 제품명 또는 SKU 검색 | |
| `--page <페이지>` | 페이지 번호 | 1 |
| `--limit <개수>` | 페이지당 항목 수 | 20 |
| `--format <형식>` | 출력 형식 (table, json, csv) | table |
| `--sort-by <필드>` | 정렬 기준 (name, sku, quantity, price, cost, created_at) | name |
| `--order <순서>` | 정렬 순서 (asc, desc) | asc |

#### 예시
```bash
# 모든 제품 조회
erp inventory list

# 카테고리별 조회
erp inventory list --category "전자제품"

# 검색 기능
erp inventory list --search "iPhone"

# JSON 형식으로 출력
erp inventory list --format json

# 가격순 정렬
erp inventory list --sort-by price --order desc
```

### inventory update - 제품 정보 수정

기존 제품의 정보를 수정합니다.

#### 사용법
```bash
erp inventory update <SKU> [옵션]
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<SKU>` | 수정할 제품의 SKU |

#### 옵션
| 옵션 | 설명 | 제약 조건 |
|------|------|----------|
| `--name <제품명>` | 새로운 제품명 | 최대 255자 |
| `--category <카테고리>` | 새로운 카테고리 | 최대 100자 |
| `--quantity <수량>` | 새로운 재고 수량 | 0 이상 (음수 불가) |
| `--price <가격>` | 새로운 가격 | 0.01 이상 (음수 불가) |
| `--cost <원가>` | 새로운 원가 | 0.01 이상 (음수 불가) |
| `--description <설명>` | 새로운 설명 | |

#### 검증 규칙
- **수량**: 음수 입력 시 명확한 에러 메시지 표시
- **가격/원가**: 음수 또는 0 입력 시 명확한 에러 메시지 표시
- **제품명**: 빈 값 불가
- **카테고리**: 빈 값 불가

#### 예시

##### 배포된 바이너리 사용
```bash
# 가격 수정
erp inventory update MBP001 --price 1899.99

# 재고 수량 수정
erp inventory update MBP001 --quantity 25

# 원가 수정
erp inventory update MBP001 --cost 1200.00

# 여러 필드 동시 수정
erp inventory update MBP001 --price 1799.99 --quantity 15 --description "할인 상품"

# 가격과 원가 동시 수정
erp inventory update MBP001 --price 1899.99 --cost 1300.00
```

##### 개발 환경에서 실행
```bash
# 가격 수정
cargo run -- inventory update MBP001 --price 1899.99

# 재고 수량 수정
cargo run -- inventory update MBP001 --quantity 25

# 원가 수정
cargo run -- inventory update MBP001 --cost 1200.00

# 여러 필드 동시 수정
cargo run -- inventory update MBP001 --price 1799.99 --quantity 15 --description "할인 상품"

# 가격과 원가 동시 수정
cargo run -- inventory update MBP001 --price 1899.99 --cost 1300.00
```

#### 에러 처리 예시
```bash
# 음수 가격 입력 시
cargo run -- inventory update TEST001 --price -50.00
# 출력: Error: Validation error: price is 가격은 음수일 수 없습니다. 0보다 큰 값을 입력하세요

# 음수 수량 입력 시
cargo run -- inventory update TEST001 --quantity -10
# 출력: Error: Validation error: quantity is 수량은 음수일 수 없습니다. 0 이상의 값을 입력하세요

# 0원 가격 입력 시
cargo run -- inventory update TEST001 --price 0.00
# 출력: Error: Validation error: price is 가격은 0보다 커야 합니다
```

### inventory remove - 제품 삭제

제품을 재고에서 완전히 삭제합니다.

#### 사용법
```bash
erp inventory remove <SKU> [옵션]
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<SKU>` | 삭제할 제품의 SKU |

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--force` | 확인 없이 삭제 | false |

#### 예시
```bash
# 제품 삭제 (확인 프롬프트 표시)
erp inventory remove MBP001

# 강제 삭제
erp inventory remove MBP001 --force
```

### inventory low-stock - 저재고 상품 조회

재고가 부족한 상품을 조회합니다.

#### 사용법
```bash
erp inventory low-stock [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--threshold <수량>` | 저재고 기준 수량 | 10 |
| `--format <형식>` | 출력 형식 | table |

#### 예시
```bash
# 기본 임계값(10개)으로 조회
erp inventory low-stock

# 임계값 5개로 조회
erp inventory low-stock --threshold 5
```

---

## 고객 관리 (customers)

고객 정보를 관리하는 명령어들입니다.

### customers add - 고객 추가

새로운 고객을 추가합니다.

#### 사용법
```bash
# 기본 방식 (전체 이름으로 입력)
erp customers add [이름] [옵션]

# 개별 방식 (성/이름 분리 입력)
erp customers add --first-name <성> --last-name <이름> [옵션]
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `[이름]` | 고객의 전체 이름 (공백으로 성과 이름 구분) 또는 회사명 |

#### 옵션
| 옵션 | 설명 | 필수 | 비고 |
|------|------|------|------|
| `--email <이메일>` | 고객 이메일 주소 | ✓ | |
| `--first-name <성>` | 고객의 성 | | 개별 입력 방식 사용 시 |
| `--last-name <이름>` | 고객의 이름 | | 개별 입력 방식 사용 시 |
| `--phone <전화번호>` | 고객 전화번호 | | 한국식 형식 지원 (010-XXXX-XXXX) |
| `--address <주소>` | 고객 주소 | | 쉼표로 구분된 형식 필요 (아래 참조) |
| `--company <회사명>` | 회사명 (기업 고객인 경우 필수) | | |
| `--tax-id <사업자등록번호>` | 사업자등록번호 (기업 고객인 경우 필수) | | |
| `--notes <메모>` | 고객 관련 메모 | | |

#### 이름 입력 방식

##### 방식 1: 전체 이름 (기존 방식)
개인 고객의 경우 공백으로 성과 이름을 구분해서 입력합니다.
```bash
erp customers add "김 철수" --email "kim@example.com"
```

##### 방식 2: 성/이름 분리 (새로운 방식)
성과 이름을 별도 옵션으로 입력합니다.
```bash
erp customers add --first-name "김" --last-name "철수" --email "kim@example.com"
```

#### 주소 입력 방식

주소는 **쉼표(,)로 구분된 형식**으로 입력해야 합니다.

##### 형식
```
--address "거리주소, 시/구, 시/도[, 우편번호, 국가]"
```

- **필수 항목** (최소 3개): 거리주소, 시/구, 시/도
- **선택 항목**: 우편번호 (기본값: "00000"), 국가 (기본값: "USA")

##### 올바른 예시
```bash
# 기본 형식 (3개 필드)
--address "테헤란로 123, 강남구, 서울시"

# 상세 형식 (5개 필드)
--address "세종대로 123, 종로구, 서울시, 03078, 대한민국"

# 다양한 형식
--address "반포대로 58, 서초구, 서울시"
--address "역삼동 123, 강남구, 서울시, 06234, 한국"
```

##### 잘못된 예시 (저장되지 않음)
```bash
# ❌ 쉼표 없이 입력 (파싱 실패)
--address "서울시 강남구 테헤란로 123"

# ❌ 필드가 2개 이하 (최소 3개 필요)
--address "서울시, 강남구"
```

##### 파싱 규칙
1. 쉼표로 구분된 문자열을 파싱
2. 각 필드의 앞뒤 공백 자동 제거
3. 최소 3개 필드 필요 (거리, 시/구, 시/도)
4. 누락된 선택 항목은 기본값 사용

#### 예시

##### 배포된 바이너리 사용
```bash
# 개인 고객 추가 (기본 방식)
erp customers add "김 철수" \
  --email "kim@example.com" \
  --phone "010-1234-5678" \
  --address "테헤란로 123, 강남구, 서울시"

# 개인 고객 추가 (개별 방식)
erp customers add \
  --first-name "이" \
  --last-name "영희" \
  --email "lee@example.com" \
  --phone "010-2345-6789" \
  --address "반포대로 58, 서초구, 서울시"

# 기업 고객 추가 (대표자 이름 개별 입력)
erp customers add \
  --first-name "박" \
  --last-name "사장" \
  --email "ceo@company.com" \
  --phone "02-1234-5678" \
  --company "ABC Corporation" \
  --tax-id "1234567890" \
  --address "역삼동 123, 강남구, 서울시" \
  --notes "주요 거래처"

# 전화번호 다양한 형식 지원
erp customers add "정 수진" \
  --email "jung@example.com" \
  --phone "(02) 1234-5678"  # 괄호 포함 형식

erp customers add "최 민호" \
  --email "choi@example.com" \
  --phone "02-3456-7890"    # 지역번호 형식
```

##### 개발 환경에서 실행
```bash
# 개인 고객 추가 (기본 방식)
cargo run -- customers add "김 철수" \
  --email "kim@example.com" \
  --phone "010-1234-5678" \
  --address "테헤란로 123, 강남구, 서울시"

# 개인 고객 추가 (개별 방식)
cargo run -- customers add \
  --first-name "이" \
  --last-name "영희" \
  --email "lee@example.com" \
  --phone "010-2345-6789" \
  --address "반포대로 58, 서초구, 서울시"

# 기업 고객 추가
cargo run -- customers add \
  --first-name "박" \
  --last-name "사장" \
  --email "ceo@company.com" \
  --phone "02-1234-5678" \
  --company "ABC Corporation" \
  --tax-id "1234567890" \
  --address "역삼동 123, 강남구, 서울시" \
  --notes "주요 거래처"
```

#### 응답
```json
{
  "status": "success",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "code": "CUST-김철-12345",
    "name": "김 철수",
    "email": "kim@example.com",
    "phone": "010-1234-5678",
    "type": "individual",
    "status": "active",
    "credit_limit": 1000.00,
    "available_credit": 1000.00,
    "company_name": null,
    "tax_id": null,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

##### 기업 고객 응답 예시
```json
{
  "status": "success",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "code": "CUST-박사-23456",
    "name": "ABC Corporation (박 사장)",
    "email": "ceo@company.com",
    "phone": "02-1234-5678",
    "type": "business",
    "status": "active",
    "credit_limit": 10000.00,
    "available_credit": 10000.00,
    "company_name": "ABC Corporation",
    "tax_id": "1234567890",
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

### customers list - 고객 목록 조회

고객 목록을 조회합니다.

#### 사용법
```bash
erp customers list [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--search <검색어>` | 고객명, 이메일, 회사명 검색 | |
| `--page <페이지>` | 페이지 번호 | 1 |
| `--limit <개수>` | 페이지당 항목 수 | 20 |
| `--format <형식>` | 출력 형식 | table |
| `--sort-by <필드>` | 정렬 기준 | name |
| `--order <순서>` | 정렬 순서 | asc |

#### 예시
```bash
# 모든 고객 조회
erp customers list

# 고객 검색
erp customers search --query "김철수"
```

### customers update - 고객 정보 수정

기존 고객의 정보를 수정합니다.

#### 사용법
```bash
erp customers update <고객ID> [옵션]
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<고객ID>` | 수정할 고객의 ID |

#### 옵션
| 옵션 | 설명 |
|------|------|
| `--name <고객명>` | 새로운 고객명 |
| `--email <이메일>` | 새로운 이메일 |
| `--phone <전화번호>` | 새로운 전화번호 |
| `--address <주소>` | 새로운 주소 |
| `--company <회사명>` | 새로운 회사명 |
| `--notes <메모>` | 새로운 메모 |

### customers delete - 고객 삭제

고객을 삭제합니다.

#### 사용법
```bash
erp customers delete <고객ID> [옵션]
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<고객ID>` | 삭제할 고객의 ID |

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--force` | 확인 없이 삭제 | false |

### customers search - 고객 검색

고객을 검색합니다.

#### 사용법
```bash
erp customers search <검색어> [옵션]
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<검색어>` | 검색할 키워드 |

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|--------|
| `--field <필드>` | 검색 필드 (name, email, phone) | 모든 필드 |
| `--format <형식>` | 출력 형식 | table |

#### 예시

##### 배포된 바이너리 사용
```bash
# 모든 필드에서 검색
erp customers search "김철수"

# 이름 필드에서만 검색
erp customers search "김철수" --field name

# 이메일 필드에서만 검색
erp customers search "kim@example.com" --field email

# 전화번호 필드에서만 검색
erp customers search "010-1234-5678" --field phone
```

##### 개발 환경에서 실행
```bash
# 모든 필드에서 검색
cargo run -- customers search "김철수"

# 이름 필드에서만 검색
cargo run -- customers search "김철수" --field name

# 이메일 필드에서만 검색
cargo run -- customers search "kim@example.com" --field email

# 전화번호 필드에서만 검색
cargo run -- customers search "010-1234-5678" --field phone
```

---

## 영업 관리 (sales)

주문과 영업을 관리하는 명령어들입니다.

### sales create-order - 주문 생성

새로운 주문을 생성합니다.

#### 사용법
```bash
erp sales create-order [옵션]
```

#### 옵션
| 옵션 | 설명 | 필수 |
|------|------|------|
| `--customer-id <ID>` | 고객 ID | ✓ |
| `--product-sku <SKU>` | 제품 SKU | ✓ |
| `--quantity <수량>` | 주문 수량 | ✓ |
| `--notes <메모>` | 주문 메모 | |

#### 예시
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

### sales list-orders - 주문 목록 조회

주문 목록을 조회합니다.

#### 사용법
```bash
erp sales list-orders [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--status <상태>` | 주문 상태 필터 | 모든 상태 |
| `--customer <ID>` | 특정 고객의 주문만 조회 | |
| `--from-date <날짜>` | 시작 날짜 (YYYY-MM-DD) | |
| `--to-date <날짜>` | 종료 날짜 (YYYY-MM-DD) | |
| `--page <페이지>` | 페이지 번호 | 1 |
| `--limit <개수>` | 페이지당 항목 수 | 20 |
| `--format <형식>` | 출력 형식 (table, json, csv) | table |

#### 주문 상태
- `draft`: 임시저장
- `pending`: 대기중
- `confirmed`: 확인됨
- `processing`: 처리중
- `shipped`: 배송됨
- `delivered`: 배송완료
- `cancelled`: 취소됨
- `returned`: 반품됨

#### 예시
```bash
# 모든 주문 조회
erp sales list-orders

# 대기중인 주문만 조회
erp sales list-orders --status pending

# 특정 기간 주문 조회
erp sales list-orders --from-date "2024-01-01" --to-date "2024-01-31"

# 특정 고객의 주문 조회
erp sales list-orders --customer "550e8400-e29b-41d4-a716-446655440000"

# JSON 형식으로 출력
erp sales list-orders --format json

# CSV 형식으로 출력
erp sales list-orders --format csv --limit 100
```

### sales update-order - 주문 상태 변경

주문의 상태를 변경합니다.

#### 사용법
```bash
erp sales update-order <주문ID> [옵션]
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<주문ID>` | 수정할 주문의 ID |

#### 옵션
| 옵션 | 설명 | 필수 |
|------|------|------|
| `--status <상태>` | 새로운 주문 상태 | ✓ |
| `--notes <메모>` | 상태 변경 메모 | |

#### 예시
```bash
# 주문 상태 변경
erp sales update-order "550e8400-e29b-41d4-a716-446655440001" --status "shipped"

# 메모와 함께 상태 변경
erp sales update-order "550e8400-e29b-41d4-a716-446655440001" \
  --status "delivered" \
  --notes "고객이 수령 확인함"
```

### sales generate-invoice - 인보이스 생성

주문에 대한 인보이스를 생성합니다.

#### 사용법
```bash
erp sales generate-invoice <주문ID> [옵션]
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<주문ID>` | 인보이스를 생성할 주문의 ID |

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--output <파일경로>` | 출력 파일 경로 | 자동 생성 |
| `--format <형식>` | 인보이스 형식 (pdf, json) | pdf |

#### 예시
```bash
# 기본 인보이스 생성
erp sales generate-invoice "550e8400-e29b-41d4-a716-446655440001"

# 특정 경로에 저장
erp sales generate-invoice "550e8400-e29b-41d4-a716-446655440001" \
  --output "/path/to/invoice.pdf"
```

---

## 보고서 (reports)

다양한 비즈니스 보고서를 생성하는 명령어들입니다.

### reports sales-summary - 매출 요약 보고서

매출 요약 보고서를 생성합니다.

#### 사용법
```bash
erp reports sales-summary [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--period <기간>` | 보고서 기간 (daily, weekly, monthly, yearly) | monthly |
| `--from <날짜>` | 시작 날짜 (YYYY-MM-DD) | |
| `--to <날짜>` | 종료 날짜 (YYYY-MM-DD) | |
| `--format <형식>` | 출력 형식 (table, json, csv, pdf) | table |
| `--output <파일경로>` | 출력 파일 경로 | |

#### 예시
```bash
# 월별 매출 요약
erp reports sales-summary --period monthly

# 특정 기간 매출 요약
erp reports sales-summary --from "2024-01-01" --to "2024-01-31"

# CSV 파일로 저장
erp reports sales-summary --period monthly --format csv --output "sales_2024_01.csv"
```

### reports inventory-status - 재고 상태 보고서

현재 재고 상태 보고서를 생성합니다.

#### 사용법
```bash
erp reports inventory-status [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--category <카테고리>` | 특정 카테고리만 포함 | 모든 카테고리 |
| `--low-stock-only` | 저재고 상품만 포함 | false |
| `--threshold <수량>` | 저재고 기준 수량 | 10 |
| `--format <형식>` | 출력 형식 | table |
| `--output <파일경로>` | 출력 파일 경로 | |

#### 예시
```bash
# 전체 재고 상태
erp reports inventory-status

# 저재고 상품만
erp reports inventory-status --low-stock-only --threshold 5

# JSON 형식으로 저장
erp reports inventory-status --format json --output "inventory_status.json"
```

### reports customer-analysis - 고객 분석 보고서

고객 분석 보고서를 생성합니다.

#### 사용법
```bash
erp reports customer-analysis [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--top <개수>` | 상위 고객 수 | 10 |
| `--metric <지표>` | 분석 지표 (revenue, orders, frequency) | revenue |
| `--period <기간>` | 분석 기간 (monthly, quarterly, yearly) | yearly |
| `--format <형식>` | 출력 형식 | table |
| `--output <파일경로>` | 출력 파일 경로 | |

#### 예시
```bash
# 상위 10명 매출 고객
erp reports customer-analysis --top 10 --metric revenue

# 주문 빈도 기준 분석
erp reports customer-analysis --metric frequency --period quarterly
```

### reports financial-overview - 재무 개요 보고서

종합적인 재무 개요 보고서를 생성합니다.

#### 사용법
```bash
erp reports financial-overview [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--period <기간>` | 보고서 기간 | monthly |
| `--include-charts` | 차트 포함 여부 | false |
| `--format <형식>` | 출력 형식 | table |
| `--export <형식>` | 내보내기 형식 (csv, pdf, excel) | |
| `--output <파일경로>` | 출력 파일 경로 | |

#### 예시
```bash
# 기본 재무 개요
erp reports financial-overview

# PDF로 내보내기
erp reports financial-overview --export pdf --output "financial_overview.pdf"

# 차트 포함 월별 보고서
erp reports financial-overview --period monthly --include-charts
```

---

## 설정 관리 (config)

시스템 설정을 관리하는 명령어들입니다.

### config get - 설정 조회

특정 설정값을 조회합니다.

#### 사용법
```bash
erp config get <설정키>
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<설정키>` | 조회할 설정의 키 (예: database.url) |

#### 예시
```bash
# 데이터베이스 URL 조회
erp config get database.url

# JWT 시크릿 조회
erp config get auth.jwt_secret
```

### config set - 설정 값 변경

설정값을 변경합니다.

#### 사용법
```bash
erp config set <설정키> <값>
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<설정키>` | 변경할 설정의 키 |
| `<값>` | 새로운 설정값 |

#### 예시
```bash
# 통화 설정
erp config set currency "KRW"

# 타임존 설정
erp config set timezone "Asia/Seoul"

# 로그 레벨 설정
erp config set logging.level "debug"

# 세금률 설정 (백분율)
erp config set tax_rate "10.0"   # 10% 부가세
erp config set tax_rate "5.0"    # 5% 세율
erp config set tax_rate "0.0"    # 세금 없음
```

### config list - 설정 목록

모든 설정을 나열합니다.

#### 사용법
```bash
erp config list [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--format <형식>` | 출력 형식 (table, json, yaml) | table |
| `--filter <패턴>` | 설정 키 필터 패턴 | |

#### 예시
```bash
# 모든 설정 조회
erp config list

# 데이터베이스 관련 설정만
erp config list --filter "database.*"

# JSON 형식으로 출력
erp config list --format json
```

### config path - 설정 파일 경로 표시

현재 사용 중인 설정 파일의 경로를 표시합니다.

#### 사용법
```bash
erp config path
```

### config reset - 설정 초기화

설정을 기본값으로 초기화합니다.

#### 사용법
```bash
erp config reset [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--confirm` | 확인 없이 초기화 | false |

#### 예시
```bash
# 설정 초기화 (확인 프롬프트 표시)
erp config reset

# 강제 초기화
erp config reset --confirm
```

### 세금률 관리

시스템의 세금률은 `tax_rate` 설정을 통해 관리됩니다. 이 설정은 모든 판매 주문에 자동으로 적용됩니다.

#### 사용법
```bash
# 세금률 조회
erp config get tax_rate

# 세금률 설정 (백분율로 입력)
erp config set tax_rate "<세율>"
```

#### 세금률 설정 예시

##### 배포된 바이너리 사용
```bash
# 한국 부가세 10% 설정
erp config set tax_rate "10.0"

# 미국 판매세 8.25% 설정
erp config set tax_rate "8.25"

# 세금 면제 (0% 설정)
erp config set tax_rate "0.0"

# 현재 세금률 확인
erp config get tax_rate
```

##### 개발 환경에서 실행
```bash
# 한국 부가세 10% 설정
cargo run -- config set tax_rate "10.0"

# 미국 판매세 8.25% 설정
cargo run -- config set tax_rate "8.25"

# 세금 면제 (0% 설정)
cargo run -- config set tax_rate "0.0"

# 현재 세금률 확인
cargo run -- config get tax_rate
```

#### 세금률 적용 범위
- **자동 적용**: 모든 새로운 주문에 자동으로 적용
- **즉시 반영**: 설정 변경 후 바로 다음 주문부터 적용
- **기존 주문**: 이미 생성된 주문의 세금률은 변경되지 않음
- **유효 범위**: 0.0% ~ 100.0% (포함)

#### 기본값
- 설정되지 않은 경우 기본값: **10.0%** (한국 부가세)

#### 주문 생성 시 세금 계산 예시
```bash
# 세금률 10%로 설정
erp config set tax_rate "10.0"

# 주문 생성 (₩150,000 상품)
erp sales create-order --customer-id CUST-12345678 --product-sku SKU-PRODUCT --quantity 1

# 결과:
# 소계: ₩150,000
# 세금: ₩15,000 (150,000 × 10%)
# 총액: ₩165,000
```

---

## 마이그레이션 (migrate)

데이터베이스 마이그레이션을 관리하는 명령어들입니다.

### migrate init - 데이터베이스 초기화

처음 실행 시 데이터베이스를 초기화합니다.

#### 사용법
```bash
erp migrate init [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--force` | 기존 데이터베이스 덮어쓰기 | false |

### migrate up - 마이그레이션 실행

데이터베이스를 최신 상태로 업데이트합니다.

#### 사용법
```bash
erp migrate up [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--steps <개수>` | 실행할 마이그레이션 단계 수 | 모든 단계 |

### migrate down - 마이그레이션 롤백

마이그레이션을 롤백합니다.

#### 사용법
```bash
erp migrate down [옵션]
```

#### 옵션
| 옵션 | 설명 | 기본값 |
|------|------|-------|
| `--steps <개수>` | 롤백할 단계 수 | 1 |

### migrate status - 마이그레이션 상태 확인

현재 마이그레이션 상태를 확인합니다.

#### 사용법
```bash
erp migrate status
```

### migrate generate - 새 마이그레이션 파일 생성

새로운 마이그레이션 파일을 생성합니다.

#### 사용법
```bash
erp migrate generate <마이그레이션명>
```

#### 필수 인수
| 인수 | 설명 |
|------|------|
| `<마이그레이션명>` | 생성할 마이그레이션의 이름 |

#### 예시
```bash
# 새 테이블 마이그레이션 생성
erp migrate generate create_products_table

# 컬럼 추가 마이그레이션 생성
erp migrate generate add_description_to_products
```

### migrate test - 데이터베이스 연결 테스트

데이터베이스 연결을 테스트합니다.

#### 사용법
```bash
erp migrate test
```

---

## 응답 형식

### 성공 응답
```json
{
  "status": "success",
  "data": {
    // 응답 데이터
  },
  "meta": {
    "total": 100,
    "page": 1,
    "limit": 20,
    "has_next": true
  }
}
```

### 에러 응답
```json
{
  "status": "error",
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "유효하지 않은 입력입니다.",
    "details": {
      "field": "email",
      "reason": "잘못된 이메일 형식"
    }
  }
}
```

### 출력 형식 옵션

#### Table (기본값)
가독성이 좋은 표 형식으로 출력합니다.

```bash
erp inventory list --format table
```

#### JSON
프로그래밍적 처리에 적합한 JSON 형식입니다.

```bash
erp inventory list --format json
```

#### CSV
스프레드시트 프로그램에서 사용할 수 있는 CSV 형식입니다.

```bash
erp inventory list --format csv
```

#### YAML
사람이 읽기 쉬운 YAML 형식입니다.

```bash
erp config list --format yaml
```

---

## 에러 코드

### 일반 에러 코드

| 코드 | 설명 |
|------|------|
| `VALIDATION_ERROR` | 입력 검증 실패 |
| `NOT_FOUND` | 리소스를 찾을 수 없음 |
| `ALREADY_EXISTS` | 이미 존재하는 리소스 |
| `UNAUTHORIZED` | 인증 실패 |
| `FORBIDDEN` | 권한 부족 |
| `DATABASE_ERROR` | 데이터베이스 오류 |
| `INTERNAL_ERROR` | 내부 서버 오류 |

### 모듈별 에러 코드

#### 재고 관리 (INVENTORY_*)
- `INVENTORY_SKU_EXISTS`: 이미 존재하는 SKU
- `INVENTORY_INSUFFICIENT_STOCK`: 재고 부족
- `INVENTORY_INVALID_QUANTITY`: 잘못된 수량 (1 ~ 2,147,483,647 범위 외)
- `INVENTORY_INVALID_PRICE`: 잘못된 가격 (0.01 ~ 99,999,999,999,999.99 범위 외)
- `INVENTORY_INVALID_COST`: 잘못된 원가 (0.01 ~ 99,999,999,999,999.99 범위 외)

#### 고객 관리 (CUSTOMER_*)
- `CUSTOMER_EMAIL_EXISTS`: 이미 존재하는 이메일
- `CUSTOMER_INVALID_EMAIL`: 잘못된 이메일 형식
- `CUSTOMER_HAS_ORDERS`: 주문이 있는 고객 삭제 불가

#### 영업 관리 (SALES_*)
- `SALES_INVALID_STATUS`: 잘못된 주문 상태
- `SALES_ORDER_CANCELLED`: 취소된 주문 수정 불가
- `SALES_INSUFFICIENT_INVENTORY`: 재고 부족으로 주문 불가

### 에러 처리 예시

```bash
# 에러 발생 시 상세 정보 확인
erp inventory add "Test Product" --sku "EXISTING_SKU" --quantity 10 --price 99.99

# 출력:
# Error: INVENTORY_SKU_EXISTS
# Message: SKU 'EXISTING_SKU'는 이미 존재합니다.
# Details: 다른 SKU를 사용하거나 기존 제품을 수정하세요.
```

---

## 환경 변수

시스템에서 사용하는 주요 환경 변수들입니다.

| 변수명 | 설명 | 기본값 |
|--------|------|--------|
| `DATABASE_URL` | 데이터베이스 연결 문자열 | `sqlite://erp.db` |
| `REDIS_URL` | Redis 연결 문자열 | |
| `JWT_SECRET` | JWT 토큰 비밀키 | |
| `LOG_LEVEL` | 로그 레벨 | `info` |
| `ERP_CONFIG_PATH` | 설정 파일 경로 | |
| `ERP_ENV` | 실행 환경 (development, production) | `development` |

### 예시 환경 변수 설정

```bash
# Linux/macOS
export DATABASE_URL="postgresql://user:password@localhost/erp_db"
export JWT_SECRET="your-secret-key-here"
export LOG_LEVEL="debug"

# Windows (PowerShell)
$env:DATABASE_URL="postgresql://user:password@localhost/erp_db"
$env:JWT_SECRET="your-secret-key-here"
$env:LOG_LEVEL="debug"
```

---

이 문서는 ERP CLI 시스템의 모든 API 기능을 다룹니다. 추가 정보가 필요하거나 문제가 발생한 경우, 개발 가이드나 사용자 가이드를 참조하시기 바랍니다.