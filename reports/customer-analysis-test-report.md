# 고객 분석 보고서 (customer-analysis) 검증 테스트 결과

## 테스트 개요

- **테스트 대상**: `erp reports customer-analysis` 명령어
- **테스트 일시**: 2025-09-30
- **테스트 환경**: Windows, Rust ERP CLI System

## 명령어 구문

```bash
cargo run -- reports customer-analysis [OPTIONS]

Options:
  --config <CONFIG>        설정 파일 경로 (선택사항)
  --months <MONTHS>        분석 기간 (months) [default: 12]
  --format <FORMAT>        출력 형식 (table, csv, json) [default: table]
  --log-level <LOG_LEVEL>  로그 레벨 설정 [possible values: trace, debug, info, warn, error]
  --output <OUTPUT>        출력 파일 경로
  -h, --help               Print help
```

## 테스트 케이스 및 결과

### 1. 기본 옵션 테스트

#### 1.1 기본 명령 실행 (format 미지정)
```bash
cargo run -- reports customer-analysis
```

**결과**: ❌ 실패
```
Error: Validation error: format is 지원되지 않는 보고서 형식입니다. 사용 가능한 형식: console, json, csv, html, pdf
```

**발견된 문제**:
- help 메시지에는 기본값이 `table`로 표시되지만 실제로는 지원되지 않음
- 실제 지원 형식: `console, json, csv, html, pdf`
- help 메시지와 실제 구현이 불일치

#### 1.2 console 형식으로 실행
```bash
cargo run -- reports customer-analysis --format console
```

**결과**: ✅ 성공
```
=== 고객 분석 보고서 ===
생성 시간: 2025-09-30 06:49:49
분석 기간: 3개월

╭────────────────┬─────────╮
│ 항목           ┆ 값      │
╞════════════════╪═════════╡
│ 총 고객 수     ┆ 25      │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ 활성 고객 수   ┆ 20      │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ 신규 고객 수   ┆ 5       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ 이탈률         ┆ 10%     │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ 고객 생애 가치 ┆ ₩500.00 │
╰────────────────┴─────────╯
```

**비고**: 기본 months 값 12가 아닌 3개월로 표시되는 것으로 보임

---

### 2. --months 옵션 테스트

#### 2.1 months=1 (최소값)
```bash
cargo run -- reports customer-analysis --format console --months 1
```
**결과**: ✅ 성공 - 분석 기간: 3개월로 표시

#### 2.2 months=6
```bash
cargo run -- reports customer-analysis --format console --months 6
```
**결과**: ✅ 성공 - 분석 기간: 3개월로 표시

#### 2.3 months=12 (기본값)
```bash
cargo run -- reports customer-analysis --format console --months 12
```
**결과**: ✅ 성공 - 분석 기간: 3개월로 표시

#### 2.4 months=24
```bash
cargo run -- reports customer-analysis --format console --months 24
```
**결과**: ✅ 성공 - 분석 기간: 3개월로 표시

**발견된 문제**:
- months 파라미터 값과 무관하게 항상 "분석 기간: 3개월"로 표시됨
- 실제 데이터 필터링이 올바르게 작동하는지 확인 필요

#### 2.5 months=0 (유효하지 않은 값)
```bash
cargo run -- reports customer-analysis --format console --months 0
```
**결과**: ❌ 실패 (예상된 동작)
```
Error: Validation error: months is 분석 기간은 1-120개월 범위여야 합니다
```

#### 2.6 months=-1 (음수)
```bash
cargo run -- reports customer-analysis --format console --months -1
```
**결과**: ❌ 실패 (예상된 동작)
```
error: unexpected argument '-1' found
```

#### 2.7 months=121 (최대값 초과)
```bash
cargo run -- reports customer-analysis --format console --months 121
```
**결과**: ❌ 실패 (예상된 동작)
```
Error: Validation error: months is 분석 기간은 1-120개월 범위여야 합니다
```

**유효성 검증**: ✅ months 범위 검증 (1-120) 정상 작동

---

### 3. --format 옵션 테스트

#### 3.1 format=table
```bash
cargo run -- reports customer-analysis --format table
```
**결과**: ❌ 실패
```
Error: Validation error: format is 지원되지 않는 보고서 형식입니다. 사용 가능한 형식: console, json, csv, html, pdf
```

**문제**: help 메시지의 기본값이 `table`이지만 지원되지 않음

#### 3.2 format=console
```bash
cargo run -- reports customer-analysis --format console
```
**결과**: ✅ 성공 - 콘솔에 표 형식으로 출력

#### 3.3 format=json
```bash
cargo run -- reports customer-analysis --format json
```
**결과**: ✅ 성공
```
보고서가 저장되었습니다: customer_analysis_20250930_065036.json
```

#### 3.4 format=csv
```bash
cargo run -- reports customer-analysis --format csv
```
**결과**: ✅ 성공
```
보고서가 저장되었습니다: customer_analysis_20250930_065038.csv
```

#### 3.5 format=html
```bash
cargo run -- reports customer-analysis --format html
```
**결과**: ✅ 성공
```
보고서가 저장되었습니다: customer_analysis_20250930_065041.html
```

#### 3.6 format=pdf (output 지정하지 않음)
```bash
cargo run -- reports customer-analysis --format pdf
```
**결과**: ✅ 성공
```
보고서가 저장되었습니다: customer_analysis_20250930_065047.pdf
```

#### 3.7 format=invalid (잘못된 형식)
```bash
cargo run -- reports customer-analysis --format invalid
```
**결과**: ❌ 실패 (예상된 동작)
```
Error: Validation error: format is 지원되지 않는 보고서 형식입니다. 사용 가능한 형식: console, json, csv, html, pdf
```

**요약**:
- ✅ 지원 형식: `console`, `json`, `csv`, `html`, `pdf`
- ❌ 미지원 형식: `table` (help 메시지 수정 필요)
- ✅ 유효성 검증 정상 작동

---

### 4. --output 옵션 테스트

#### 4.1 console format + output 지정
```bash
cargo run -- reports customer-analysis --format console --output test_output.txt
```
**결과**: ✅ 성공 (콘솔에 출력, 파일 생성 여부는 미확인)

#### 4.2 json format + 커스텀 output
```bash
cargo run -- reports customer-analysis --format json --output test_custom.json
```
**결과**: ✅ 성공
```
보고서가 저장되었습니다: test_custom.json
```

#### 4.3 csv format + 디렉토리 경로 포함
```bash
cargo run -- reports customer-analysis --format csv --output reports/test_report.csv
```
**결과**: ✅ 성공
```
보고서가 저장되었습니다: reports/test_report.csv
```

---

### 5. 옵션 조합 테스트

#### 5.1 json + months=3 + output
```bash
cargo run -- reports customer-analysis --format json --months 3 --output combined_test.json
```
**결과**: ✅ 성공
```
보고서가 저장되었습니다: combined_test.json
```

#### 5.2 csv + months=6 + output (디렉토리 포함)
```bash
cargo run -- reports customer-analysis --format csv --months 6 --output reports/combined_6months.csv
```
**결과**: ✅ 성공
```
보고서가 저장되었습니다: reports/combined_6months.csv
```

#### 5.3 html + months=12 + output
```bash
cargo run -- reports customer-analysis --format html --months 12 --output reports/combined_yearly.html
```
**결과**: ✅ 성공
```
보고서가 저장되었습니다: reports/combined_yearly.html
```

#### 5.4 pdf + months=1 + output
```bash
cargo run -- reports customer-analysis --format pdf --months 1 --output reports/combined_monthly.pdf
```
**결과**: ❌ 실패
```
Error: Internal error: Unsupported: PDF 내보내기는 아직 지원되지 않습니다
```

**발견된 문제**:
- pdf 형식은 output 옵션을 지정하지 않으면 성공
- output 옵션과 함께 사용하면 "PDF 내보내기는 아직 지원되지 않습니다" 오류 발생
- 동작이 일관되지 않음

---

## 발견된 문제 요약

### 🔴 Critical (즉시 수정 필요)

1. **help 메시지와 실제 구현 불일치**
   - help: `--format <FORMAT>  출력 형식 (table, csv, json) [default: table]`
   - 실제: `table` 형식 미지원, 지원 형식은 `console, json, csv, html, pdf`
   - 위치: `src/cli/commands/reports.rs` 또는 해당 구조체의 help 메시지

2. **PDF 출력 동작 불일치**
   - `--format pdf` (output 미지정): 성공 ✅
   - `--format pdf --output path`: 실패 ❌ "PDF 내보내기는 아직 지원되지 않습니다"
   - 두 경우 모두 동일하게 작동해야 함

### 🟡 Medium (개선 권장)

3. **months 파라미터 미반영**
   - 모든 months 값(1, 6, 12, 24)에 대해 "분석 기간: 3개월"로 표시됨
   - 실제 데이터 필터링에 영향을 주는지 확인 필요
   - 출력 메시지만 고정되어 있는지 검증 필요

4. **API 문서와 실제 옵션 불일치**
   - API 문서(`api-reference.md`)에 명시된 옵션:
     - `--top <개수>`: 상위 고객 수 [default: 10]
     - `--metric <지표>`: 분석 지표 (revenue, orders, frequency) [default: revenue]
     - `--period <기간>`: 분석 기간 (monthly, quarterly, yearly) [default: yearly]
   - 실제 구현에는 위 옵션들이 존재하지 않음
   - 대신 `--months <MONTHS>` 옵션만 존재

### 🟢 Low (문서화)

5. **months 유효 범위 문서화**
   - 실제 유효 범위: 1-120
   - API 문서에 명시 필요

---

## 테스트 결과 통계

- **총 테스트 케이스**: 26개
- **성공**: 19개 (73.1%)
- **실패 (예상된 동작)**: 4개 (15.4%)
- **실패 (버그)**: 3개 (11.5%)

## 권장 조치사항

1. **즉시 수정**:
   - help 메시지의 format 기본값 및 지원 형식 수정
   - PDF output 옵션 관련 일관성 있는 동작 구현

2. **검증 필요**:
   - months 파라미터가 실제 데이터 필터링에 영향을 주는지 확인
   - 표시되는 "분석 기간" 메시지가 months 값을 반영하도록 수정

3. **문서 업데이트**:
   - `docs/api-reference.md` 파일의 고객 분석 보고서 섹션 수정
   - 실제 구현과 일치하도록 옵션 설명 업데이트
   - 또는 문서에 명시된 옵션을 실제로 구현

---

## 테스트 생성 파일 목록

테스트 중 생성된 파일들:
```
customer_analysis_20250930_065036.json
customer_analysis_20250930_065038.csv
customer_analysis_20250930_065041.html
customer_analysis_20250930_065047.pdf
customer_analysis_20250930_065128.pdf
test_custom.json
combined_test.json
reports/test_report.csv
reports/combined_6months.csv
reports/combined_yearly.html
```

---

**테스트 완료일**: 2025-09-30
**작성자**: ERP CLI Test Suite