# Inventory Add 명령어 검증 보고서

**작성일**: 2025-09-30
**검증 범위**: `inventory add` 명령어의 모든 경우의 수
**문서 참조**: docs/api-reference.md

## 개요

`inventory add` 명령어의 동작을 검증하기 위해 다양한 입력 조합을 테스트했습니다. 필수 옵션, 선택 옵션, 경계값, 예외 상황 등을 포함한 종합적인 테스트를 수행했습니다.

## 테스트 결과 요약

- **총 테스트 케이스**: 25개
- **성공 케이스**: 11개
- **실패 케이스**: 14개
- **발견된 이슈**: 4개

---

## ✅ 성공 케이스

### 1. 원가 직접 지정
```bash
cargo run -- inventory add "Test Product 2" --sku "TEST002" --category "전자제품" --quantity 50 --price 799.99 --cost 500.00 --description "원가 직접 지정 테스트"
```
**결과**: ✅ 성공
**비고**: 원가 직접 지정, 마진율 59.9%

---

### 2. 원가 자동 계산
```bash
cargo run -- inventory add "Test Product 3" --sku "TEST003" --category "가전제품" --quantity 25 --price 1299.99 --description "카테고리와 설명 포함"
```
**결과**: ✅ 성공
**비고**: 원가 자동 계산 (가격의 70%), 마진율 42.8%

---

### 3. 최소 필수 옵션만 사용
```bash
cargo run -- inventory add "Minimal Product" --sku "TEST004" --quantity 5 --price 99.99
```
**결과**: ✅ 성공
**비고**: 카테고리 기본값 "general", 원가 자동 계산

---

### 4. SKU 자동 생성
```bash
cargo run -- inventory add "Test Product" --quantity 10 --price 100
```
**결과**: ✅ 성공
**비고**: SKU 자동 생성 (SKU-269C4CA7)

---

### 5. 긴 제품명
```bash
cargo run -- inventory add "Very Long Name Product That Has More Than Fifty Characters In Its Name" --sku "TEST014" --quantity 10 --price 100
```
**결과**: ✅ 성공
**비고**: 50자 이상 제품명 허용

---

### 6. 한글 제품명
```bash
cargo run -- inventory add "한글제품명" --sku "TEST016" --quantity 10 --price 100
```
**결과**: ✅ 성공
**비고**: 한글 제품명 지원

---

### 7. 원가가 가격보다 높은 경우
```bash
cargo run -- inventory add "High Cost" --sku "TEST012" --quantity 10 --price 100 --cost 200
```
**결과**: ✅ 성공
**비고**: 마진 -100.00원, 마진율 -50.0%

---

### 8. 소수점 3자리 가격
```bash
cargo run -- inventory add "Decimal Price" --sku "TEST019" --quantity 10 --price 99.999
```
**결과**: ✅ 성공
**비고**: 소수점 3자리 입력 시 자동 반올림 (99.99)

---

### 9. 원가 0원
```bash
cargo run -- inventory add "Zero Cost" --sku "TEST020" --quantity 10 --price 100 --cost 0
```
**결과**: ✅ 성공
**비고**: 원가 0원 허용, 마진 100.00원, 마진율 0.0% (⚠️ 계산 오류)

---

### 10. 원가와 가격이 동일
```bash
cargo run -- inventory add "Same Cost Price" --sku "TEST021" --quantity 10 --price 100 --cost 100
```
**결과**: ✅ 성공
**비고**: 원가=가격, 마진 0원, 마진율 0.0%

---

### 11. 카테고리 미지정
```bash
cargo run -- inventory add "No Category" --sku "TEST022" --quantity 10 --price 100
```
**결과**: ✅ 성공
**비고**: 카테고리 기본값 "general" 자동 적용

---

## ❌ 실패 케이스

### 1. 중복 SKU (기존 데이터)
```bash
cargo run -- inventory add "Test Product 1" --sku "TEST001" --quantity 10 --price 1999.99
```
**에러**: `Error: Conflict: SKU 'TEST001' already exists`
**분류**: 중복 SKU

---

### 2. 필수 옵션 누락 (quantity, price)
```bash
cargo run -- inventory add "Test Product" --sku "TEST005"
```
**에러**: `error: the following required arguments were not provided: --quantity <QUANTITY> --price <PRICE>`
**분류**: 필수 옵션 누락

---

### 3. 필수 옵션 누락 (price)
```bash
cargo run -- inventory add "Test Product" --sku "TEST006" --quantity 10
```
**에러**: `error: the following required arguments were not provided: --price <PRICE>`
**분류**: 필수 옵션 누락

---

### 4. 빈 제품명
```bash
cargo run -- inventory add "" --sku "TEST007" --quantity 10 --price 100
```
**에러**: `Error: Validation error: name is 제품명은 비어있을 수 없습니다`
**분류**: 입력 검증 실패

---

### 5. 빈 SKU
```bash
cargo run -- inventory add "Test Product" --sku "" --quantity 10 --price 100
```
**에러**: `Error: Validation error: sku is SKU는 비어있을 수 없습니다`
**분류**: 입력 검증 실패

---

### 6. 음수 수량
```bash
cargo run -- inventory add "Negative Quantity" --sku "TEST008" --quantity -5 --price 100
```
**에러**: `error: unexpected argument '-5' found`
**분류**: clap 파서 에러 (음수를 옵션으로 인식)

---

### 7. 음수 가격
```bash
cargo run -- inventory add "Negative Price" --sku "TEST009" --quantity 10 --price -100
```
**에러**: `error: unexpected argument '-1' found`
**분류**: clap 파서 에러 (음수를 옵션으로 인식)

---

### 8. 수량 0 ⚠️
```bash
cargo run -- inventory add "Zero Quantity" --sku "TEST010" --quantity 0 --price 100
```
**에러**: `Error: Internal error: Failed to create product: error returned from database: invalid input value for enum product_status: "outofstock"`
**분류**: 데이터베이스 enum 에러
**이슈**: product_status enum에 "outofstock" 값이 없음

---

### 9. 가격 0원
```bash
cargo run -- inventory add "Zero Price" --sku "TEST011" --quantity 10 --price 0
```
**에러**: `Error: Validation error: input is Price must be greater than zero`
**분류**: 입력 검증 실패

---

### 10. 음수 원가
```bash
cargo run -- inventory add "Negative Cost" --sku "TEST013" --quantity 10 --price 100 --cost -50
```
**에러**: `error: unexpected argument '-5' found`
**분류**: clap 파서 에러

---

### 11. 특수문자 포함 SKU
```bash
cargo run -- inventory add "Special Chars" --sku "TEST-015@#$" --quantity 10 --price 100
```
**에러**: `Error: Validation error: sku is SKU는 영문, 숫자, 하이픈(-), 언더스코어(_)만 허용됩니다`
**분류**: 입력 검증 실패

---

### 12. 중복 SKU
```bash
cargo run -- inventory add "Duplicate SKU" --sku "TEST001" --quantity 10 --price 100
```
**에러**: `Error: Conflict: SKU 'TEST001' already exists`
**분류**: 중복 SKU

---

### 13. 수량 상한 초과
```bash
cargo run -- inventory add "Large Quantity" --sku "TEST017" --quantity 999999999 --price 100
```
**에러**: `Error: Validation error: quantity is 수량이 너무 큽니다 (최대: 1,000,000)`
**분류**: 입력 검증 실패

---

### 14. 가격 상한 초과 ⚠️
```bash
cargo run -- inventory add "Large Price" --sku "TEST018" --quantity 10 --price 999999999.99
```
**에러**: `Error: Internal error: Failed to create product: error returned from database: numeric field overflow`
**분류**: 데이터베이스 numeric 타입 범위 초과
**이슈**: 가격 상한 검증 누락

---

## 🐛 발견된 이슈

### Issue #1: 수량 0 입력 시 DB enum 에러
**심각도**: 🔴 High
**설명**: 수량을 0으로 입력하면 `product_status` enum에 `outofstock` 값이 없다는 DB 에러가 발생합니다.
**재현**:
```bash
cargo run -- inventory add "Zero Quantity" --sku "TEST010" --quantity 0 --price 100
```
**에러 메시지**:
```
Error: Internal error: Failed to create product: error returned from database: invalid input value for enum product_status: "outofstock"
```
**권장 수정**:
- DB enum에 `outofstock` 값 추가, 또는
- 수량 0 입력을 검증 단계에서 차단

---

### Issue #2: 음수 값 입력 시 clap 파서 에러
**심각도**: 🟡 Medium
**설명**: 음수 값을 입력하면 clap이 이를 새로운 옵션으로 인식하여 파싱 에러가 발생합니다.
**재현**:
```bash
cargo run -- inventory add "Negative Quantity" --sku "TEST008" --quantity -5 --price 100
```
**에러 메시지**:
```
error: unexpected argument '-5' found
  tip: to pass '-5' as a value, use '-- -5'
```
**권장 수정**:
- clap의 `allow_negative_numbers` 옵션 활성화, 또는
- 애플리케이션 레벨에서 음수 검증 추가

---

### Issue #3: 매우 큰 가격 입력 시 DB overflow
**심각도**: 🟡 Medium
**설명**: 데이터베이스 numeric 타입 범위를 초과하는 가격을 입력하면 DB overflow 에러가 발생합니다.
**재현**:
```bash
cargo run -- inventory add "Large Price" --sku "TEST018" --quantity 10 --price 999999999.99
```
**에러 메시지**:
```
Error: Internal error: Failed to create product: error returned from database: numeric field overflow
```
**권장 수정**:
- 가격 상한 검증 추가 (예: 최대 10,000,000.00)

---

### Issue #4: 마진율 계산 오류 (원가 0원)
**심각도**: 🟢 Low
**설명**: 원가가 0원일 때 마진율이 0.0%로 표시되지만, 실제로는 100% 마진입니다.
**재현**:
```bash
cargo run -- inventory add "Zero Cost" --sku "TEST020" --quantity 10 --price 100 --cost 0
```
**출력**:
```
│ 마진      ┆ ₩100.00 (0.0%)                       │
```
**권장 수정**:
- 마진율 계산 공식 수정: `(price - cost) / price * 100` 대신 `(price - cost) / cost * 100` 사용
- 원가 0원일 때 특별 처리 (예: "N/A" 또는 "∞")

---

## 📊 검증 통계

### 옵션별 검증 결과

| 옵션 | 필수 여부 | 자동 생성 | 검증 규칙 | 상태 |
|------|----------|----------|----------|------|
| `name` | ✓ | ✗ | 비어있지 않음 | ✅ |
| `sku` | 조건부 | ✓ | 영문, 숫자, -, _ | ✅ |
| `quantity` | ✓ | ✗ | 0-1,000,000 | ⚠️ (0 처리 오류) |
| `price` | ✓ | ✗ | > 0 | ⚠️ (상한 미검증) |
| `cost` | ✗ | ✓ (70%) | >= 0 | ⚠️ (음수 파서 오류) |
| `category` | ✗ | ✓ (general) | - | ✅ |
| `description` | ✗ | ✗ | - | ✅ |

### 에러 처리 품질

| 에러 유형 | 적절한 메시지 | 일관성 | 평가 |
|----------|-------------|--------|------|
| 필수 옵션 누락 | ✅ | ✅ | 우수 |
| 입력 검증 실패 | ✅ | ✅ | 우수 |
| 중복 SKU | ✅ | ✅ | 우수 |
| DB 에러 | ⚠️ | ⚠️ | 개선 필요 |
| 파서 에러 | ⚠️ | ⚠️ | 개선 필요 |

---

## 🎯 권장 사항

### 우선순위 High
1. **수량 0 입력 처리**: DB enum 추가 또는 검증 강화
2. **음수 값 처리**: clap 설정 또는 검증 추가

### 우선순위 Medium
3. **가격 상한 검증**: 데이터베이스 범위 내 검증 추가
4. **에러 메시지 개선**: DB 에러를 사용자 친화적 메시지로 변환

### 우선순위 Low
5. **마진율 계산 수정**: 원가 0원일 때 특별 처리

---

## 📝 결론

`inventory add` 명령어는 전반적으로 잘 동작하며, 대부분의 입력 검증이 적절하게 수행됩니다. 그러나 수량 0 처리, 음수 값 파싱, 가격 상한 검증 등 일부 경계 케이스에서 개선이 필요합니다. 특히 데이터베이스 에러가 사용자에게 직접 노출되는 것은 UX 관점에서 개선이 필요합니다.

**전체 평가**: ⭐⭐⭐⭐☆ (4/5)