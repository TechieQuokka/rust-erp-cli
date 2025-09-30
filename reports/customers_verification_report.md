# 고객 관리(Customers) 기능 검증 보고서

생성일: 2025-09-30
검증자: Claude Code
대상 버전: ERP CLI v0.1.0

---

## 목차
1. [customers list - 고객 목록 조회](#1-customers-list---고객-목록-조회)
2. [customers search - 고객 검색](#2-customers-search---고객-검색)
3. [발견된 문제점](#3-발견된-문제점)
4. [권장사항](#4-권장사항)

---

## 1. customers list - 고객 목록 조회

### 1.1 기본 조회 (옵션 없음)

**명령어:**
```bash
cargo run -- customers list
```

**결과:** ✅ **성공**
- 총 48개의 고객 데이터가 존재
- 기본적으로 페이지당 20개 항목 표시
- 1페이지/3페이지 표시
- 표시 항목: Code, Name, Email, Type, Status, Credit Limit, Balance, Available
- 테이블 형식으로 깔끔하게 출력

---

### 1.2 검색(--search) 옵션

#### 1.2.1 한글 이름 검색

**명령어:**
```bash
cargo run -- customers list --search "김"
```

**결과:** ✅ **성공**
- 3명의 고객 검색됨
- "김"이 포함된 고객명 필터링 성공
- 결과: 김 지우, SeoulK (김 철수), 김민수

#### 1.2.2 회사명 검색

**명령어:**
```bash
cargo run -- customers list --search "Company"
```

**결과:** ✅ **성공**
- 7개의 회사 고객 검색됨
- 회사명에 "Company" 포함된 항목 모두 검색
- LG전자, SK텔레콤, 그린에너지, 삼성전자, 테크스타트업, 푸드테크, 현대자동차

#### 1.2.3 이메일/코드 검색

**명령어:**
```bash
cargo run -- customers list --search "test"
```

**결과:** ✅ **성공**
- 13개의 테스트 계정 검색됨
- 이름, 이메일 등에서 "test" 키워드 검색 성공

---

### 1.3 페이지네이션(--page) 옵션

**명령어:**
```bash
cargo run -- customers list --page 2
```

**결과:** ✅ **성공**
- 2페이지 데이터 정상 표시
- 20개 항목 표시 (2페이지/3페이지)
- 페이지 번호 증가에 따른 데이터 변경 확인

---

### 1.4 페이지당 항목 수(--limit) 옵션

**명령어:**
```bash
cargo run -- customers list --limit 5
```

**결과:** ✅ **성공**
- 5개 항목만 표시
- 전체 48개 중 5개 표시 (1페이지/10페이지)
- limit 값에 따라 페이지 수 동적 계산 확인

---

### 1.5 JSON 형식(--format json) 출력

**명령어:**
```bash
cargo run -- customers list --format json --limit 2
```

**결과:** ✅ **성공**
```json
{
  "data": [
    {
      "id": "97c2b884-b71b-44ed-8c1d-1dc140d8b8be",
      "customer_code": "CUST-97c2b884",
      "first_name": "LG전자",
      "last_name": "",
      "email": "info@lg.com",
      "phone": "02-3777-1114",
      "company_name": "LG전자 Company",
      "tax_id": "1348200123",
      "customer_type": "Individual",
      "status": "Active",
      "credit_limit": "10000.00",
      "current_balance": "0",
      "available_credit": "10000.00",
      "addresses": [],
      "notes": null,
      "created_at": "2025-09-29T04:55:19.622606Z",
      "updated_at": "2025-09-29T04:55:19.622606Z"
    },
    ...
  ],
  "meta": {
    "page": 1,
    "per_page": 2,
    "total": 48,
    "total_pages": 24
  },
  "status": "success"
}
```
- 정확한 JSON 구조로 출력
- meta 정보 포함 (페이지네이션 정보)
- 프로그래밍적 처리에 적합한 형식

---

### 1.6 정렬(--sort-by, --order) 옵션

#### 1.6.1 이름 오름차순 정렬

**명령어:**
```bash
cargo run -- customers list --sort-by name --order asc --limit 5
```

**결과:** ✅ **성공**
- 알파벳/가나다 순으로 정렬
- LG전자, SK텔레콤, SeoulL 등이 앞에 표시

#### 1.6.2 이름 내림차순 정렬

**명령어:**
```bash
cargo run -- customers list --sort-by name --order desc --limit 5
```

**결과:** ✅ **성공**
- 역순으로 정렬
- 황 지민, 홍길동, 현대자동차, 한 예진, 푸드테크 순으로 표시
- 한글이 영문보다 뒤에 배치됨

---

### 1.7 복합 옵션 조합

**명령어:**
```bash
cargo run -- customers list --page 3 --limit 10
```

**결과:** ✅ **성공**
- 3페이지에 10개씩 표시
- 전체 48개 중 21~30번째 항목 표시 (3페이지/5페이지)
- 여러 옵션의 조합이 정상 작동

---

## 2. customers search - 고객 검색

### 2.1 기본 검색 (모든 필드)

#### 2.1.1 한글 이름 검색

**명령어:**
```bash
cargo run -- customers search "김철수"
```

**결과:** ❌ **실패**
- 검색 결과 없음: "No customers found for query: '김철수'"
- 데이터베이스에 "김 철수" (공백 포함) 데이터가 있으나 검색되지 않음
- **문제점**: 공백 처리 또는 부분 일치 검색 미지원

#### 2.1.2 영문 키워드 검색

**명령어:**
```bash
cargo run -- customers search "Samsung"
```

**결과:** ⚠️ **옵션 에러**
- 에러 메시지: `error: unexpected argument '--format' found`
- search 명령어가 `--format` 옵션을 지원하지 않음
- API 문서에는 `--format` 옵션이 명시되어 있으나 실제로는 지원하지 않음

---

### 2.2 특정 필드 검색

#### 2.2.1 이름 필드 검색 (--field name)

**명령어:**
```bash
cargo run -- customers search "김철수" --field name
```

**결과:** ❌ **실패**
- 검색 결과 없음
- `customers list --search "김"`으로는 검색되나 `customers search "김철수" --field name`으로는 검색 안됨
- **문제점**: 정확한 일치만 지원하거나, 공백 처리 문제

#### 2.2.2 이메일 필드 검색 (--field email)

**명령어:**
```bash
cargo run -- customers search "kim@example.com" --field email
```

**결과:** ✅ **성공**
```
╭───────────────┬─────────┬─────────────────┬───────────────┬────────────┬────────┬─────────╮
│ Code          ┆ Name    ┆ Email           ┆ Phone         ┆ Type       ┆ Status ┆ Balance │
╞═══════════════╪═════════╪═════════════════╪═══════════════╪════════════╪════════╪═════════╡
│ CUST-a1564274 ┆ 김민수  ┆ kim@example.com ┆ 010-9876-5432 ┆ individual ┆ active ┆ $0      │
╰───────────────┴─────────┴─────────────────┴───────────────┴────────────┴────────┴─────────╯

Found 1 customer(s) matching 'kim@example.com'
```
- 이메일 정확히 일치하는 검색 성공
- 1명의 고객 검색됨

#### 2.2.3 전화번호 필드 검색 (--field phone)

**명령어:**
```bash
cargo run -- customers search "010-9876-5432" --field phone
```

**결과:** ❌ **실패**
- 검색 결과 없음: "No customers found for query: '010-9876-5432'"
- 데이터베이스에 해당 전화번호가 존재하나 검색 안됨
- **문제점**: 전화번호 포맷 정규화 또는 검색 로직 문제

---

## 3. 발견된 문제점

### 3.1 customers search 명령어 문제

#### 문제 1: 한글 이름 검색 실패
- **증상**: "김철수" 검색 시 결과 없음
- **원인 추정**:
  - 데이터베이스에는 "김 철수" (공백 포함)로 저장
  - 부분 일치 검색(LIKE) 미지원 또는 공백 처리 문제
- **재현 방법**:
  ```bash
  cargo run -- customers search "김철수"
  # 결과: No customers found
  ```
- **우회 방법**: `customers list --search "김"` 사용

#### 문제 2: 전화번호 검색 실패
- **증상**: 정확한 전화번호 입력해도 검색 안됨
- **원인 추정**:
  - 전화번호 포맷 정규화 미지원
  - 데이터베이스 저장 형식과 검색 입력 형식 불일치
- **재현 방법**:
  ```bash
  cargo run -- customers search "010-9876-5432" --field phone
  # 결과: No customers found
  ```

#### 문제 3: --format 옵션 미지원
- **증상**: `--format` 옵션 사용 시 에러 발생
- **원인**: API 문서와 실제 구현 불일치
- **재현 방법**:
  ```bash
  cargo run -- customers search "Samsung" --format json
  # 에러: error: unexpected argument '--format' found
  ```
- **문서 오류**: docs/api-reference.md 564줄에 `--format <형식>` 옵션이 명시되어 있으나 실제로는 미구현

---

### 3.2 customers list vs customers search 차이점

| 기능 | customers list --search | customers search |
|------|------------------------|------------------|
| 부분 일치 검색 | ✅ 지원 | ❌ 미지원 또는 제한적 |
| 한글 검색 | ✅ 정상 작동 | ❌ 공백 처리 문제 |
| 전화번호 검색 | 테스트 안함 | ❌ 실패 |
| --format 옵션 | ✅ 지원 (json, csv, table) | ❌ 미지원 |
| 페이지네이션 | ✅ 지원 | ❌ 미지원 |
| 정렬 기능 | ✅ 지원 | ❌ 미지원 |

**결론**: `customers list --search` 명령어가 `customers search`보다 기능이 더 풍부하고 안정적

---

## 4. 권장사항

### 4.1 즉시 수정 필요 사항

1. **customers search 명령어 개선**
   - 부분 일치 검색(LIKE '%검색어%') 구현
   - 공백 정규화 처리 추가
   - 전화번호 포맷 정규화 (하이픈 제거 후 검색)

2. **API 문서 수정**
   - `customers search` 명령어에서 지원하지 않는 `--format` 옵션 제거
   - 또는 `--format` 옵션 구현

3. **검색 로직 통일**
   - `customers list --search`와 `customers search`의 검색 로직 동일하게 구현
   - 중복 코드 제거 및 공통 모듈화

---

### 4.2 테스트 범위 확장 권장

다음 항목은 이번 검증에서 테스트하지 않았으나 추가 검증 권장:

1. **customers add** - 고객 추가
   - 필수 필드 검증 (email, name)
   - 선택 필드 검증 (phone, address, company, tax_id)
   - 중복 이메일 처리
   - 한글 이름 처리
   - 전화번호 포맷 검증

2. **customers update** - 고객 정보 수정
   - 존재하지 않는 ID 처리
   - 부분 업데이트 기능
   - 이메일 중복 검증

3. **customers delete** - 고객 삭제
   - 존재하지 않는 ID 처리
   - --force 옵션 동작
   - 주문이 있는 고객 삭제 제한

---

### 4.3 코드 개선 제안

#### 제안 1: 검색 로직 개선 (src/modules/customers/repository.rs)

```rust
// 현재 추정 로직
pub async fn search_by_field(&self, query: &str, field: &str) -> Result<Vec<Customer>> {
    // 정확한 일치만 검색
    let sql = format!("SELECT * FROM customers WHERE {} = $1", field);
    // ...
}

// 개선 제안
pub async fn search_by_field(&self, query: &str, field: &str) -> Result<Vec<Customer>> {
    // 부분 일치 검색
    let pattern = format!("%{}%", query);
    let sql = format!("SELECT * FROM customers WHERE {} ILIKE $1", field);
    // ILIKE: 대소문자 구분 없는 부분 일치 검색
    // ...
}
```

#### 제안 2: 전화번호 정규화

```rust
fn normalize_phone(phone: &str) -> String {
    phone.chars()
        .filter(|c| c.is_numeric())
        .collect()
}

// 검색 시
let normalized_query = normalize_phone(query);
let sql = "SELECT * FROM customers WHERE REGEXP_REPLACE(phone, '[^0-9]', '', 'g') = $1";
```

#### 제안 3: 공백 정규화

```rust
fn normalize_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
```

---

## 5. 테스트 통계

### 5.1 customers list 명령어

| 테스트 항목 | 성공 | 실패 | 부분 성공 |
|------------|------|------|----------|
| 기본 조회 | ✅ | - | - |
| --search 옵션 | ✅ (3/3) | - | - |
| --page 옵션 | ✅ | - | - |
| --limit 옵션 | ✅ | - | - |
| --format json | ✅ | - | - |
| --sort-by 옵션 | ✅ (2/2) | - | - |
| 복합 옵션 | ✅ | - | - |
| **합계** | **10** | **0** | **0** |

**성공률: 100%**

---

### 5.2 customers search 명령어

| 테스트 항목 | 성공 | 실패 | 부분 성공 |
|------------|------|------|----------|
| 기본 검색 (한글) | - | ❌ | - |
| 기본 검색 (영문) | - | - | ⚠️ (옵션 에러) |
| --field name | - | ❌ | - |
| --field email | ✅ | - | - |
| --field phone | - | ❌ | - |
| --format 옵션 | - | ❌ | - |
| **합계** | **1** | **4** | **1** |

**성공률: 16.7%**

---

## 6. 결론

### 6.1 전체 평가

**customers list** 명령어는 모든 기능이 정상 작동하며 안정적입니다.

**customers search** 명령어는 심각한 기능 결함이 있으며, 한글 검색, 전화번호 검색이 제대로 작동하지 않습니다. API 문서와 실제 구현 간의 불일치도 발견되었습니다.

### 6.2 우선순위

**P0 (Critical):**
- customers search의 한글 이름 검색 문제
- customers search의 전화번호 검색 문제

**P1 (High):**
- API 문서와 실제 구현 불일치 (--format 옵션)
- customers list와 customers search 검색 로직 통일

**P2 (Medium):**
- 검색 성능 최적화 (인덱스 활용)
- 추가 테스트 케이스 작성

---

## 7. 참고 자료

- API 문서: docs/api-reference.md
- 소스 코드: src/cli/commands/customers.rs
- 데이터베이스: PostgreSQL (erp_db)

---

**보고서 끝**