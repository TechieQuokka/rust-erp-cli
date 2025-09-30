# ERP CLI 설치 가이드

이 문서는 ERP CLI 시스템의 설치 방법을 다양한 환경에 대해 안내합니다.

## 목차

- [시스템 요구사항](#시스템-요구사항)
- [빠른 설치](#빠른-설치)
- [상세 설치 가이드](#상세-설치-가이드)
  - [바이너리 설치](#바이너리-설치)
  - [소스에서 컴파일](#소스에서-컴파일)
  - [Docker 설치](#docker-설치)
  - [패키지 매니저 설치](#패키지-매니저-설치)
- [데이터베이스 설정](#데이터베이스-설정)
- [초기 설정](#초기-설정)
- [문제 해결](#문제-해결)

## 시스템 요구사항

### 최소 요구사항

- **운영체제**: Windows 10+, macOS 10.15+, Linux (Ubuntu 18.04+, CentOS 7+)
- **메모리**: 최소 512MB RAM
- **디스크 공간**: 최소 100MB
- **네트워크**: 인터넷 연결 (초기 설치 및 업데이트용)

### 권장 요구사항

- **운영체제**: Windows 11, macOS 12+, Linux (Ubuntu 20.04+)
- **메모리**: 2GB+ RAM
- **디스크 공간**: 1GB+ (데이터베이스 포함)
- **CPU**: 2코어 이상

### 데이터베이스 (선택사항)

- **개발환경**: SQLite (자동 설치)
- **프로덕션환경**: PostgreSQL 13+ (권장)
- **캐시**: Redis 6+ (선택사항)

## 빠른 설치

### Linux/macOS

```bash
# 원클릭 설치 스크립트
curl -fsSL https://raw.githubusercontent.com/example/erp-cli/main/scripts/install.sh | bash

# 또는 wget 사용
wget -qO- https://raw.githubusercontent.com/example/erp-cli/main/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
# PowerShell 관리자 권한으로 실행
iwr -useb https://raw.githubusercontent.com/example/erp-cli/main/scripts/install.ps1 | iex
```

### 설치 확인

```bash
erp --version
erp --help
```

## 상세 설치 가이드

### 바이너리 설치

#### 1. 릴리스 다운로드

[GitHub Releases 페이지](https://github.com/example/erp-cli/releases)에서 플랫폼에 맞는 바이너리를 다운로드합니다.

#### 2. 플랫폼별 설치

**Linux/macOS:**

```bash
# 다운로드한 바이너리를 실행 가능하게 만들기
chmod +x erp-linux-amd64  # 또는 erp-macos-amd64

# PATH에 추가
sudo mv erp-linux-amd64 /usr/local/bin/erp

# 또는 홈 디렉토리에 설치
mkdir -p ~/.local/bin
mv erp-linux-amd64 ~/.local/bin/erp
export PATH="$HOME/.local/bin:$PATH"
```

**Windows:**

```cmd
# 다운로드한 erp-windows-amd64.exe를 원하는 위치에 저장
# 예: C:\Program Files\ERP\erp.exe

# PATH에 추가 (시스템 환경 변수 또는 사용자 환경 변수)
setx PATH "%PATH%;C:\Program Files\ERP"
```

### 소스에서 컴파일

#### 1. Rust 설치

```bash
# Rust 설치 (rustup 사용)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### 2. 의존성 설치

**Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install -y build-essential libssl-dev libpq-dev pkg-config
```

**CentOS/RHEL:**

```bash
sudo yum groupinstall -y "Development Tools"
sudo yum install -y openssl-devel postgresql-devel pkg-config
```

**macOS:**

```bash
# Homebrew 필요
brew install postgresql openssl pkg-config
```

**Windows:**

```cmd
# Visual Studio Build Tools 설치 필요
# https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

# PostgreSQL 개발 라이브러리 (선택사항)
```

#### 3. 컴파일 및 설치

```bash
# 소스 코드 클론
git clone https://github.com/example/erp-cli.git
cd erp-cli

# 릴리스 빌드
cargo build --release

# 바이너리 설치
sudo cp target/release/erp /usr/local/bin/
# 또는 Windows에서
copy target\release\erp.exe "C:\Program Files\ERP\"
```

### Docker 설치

#### 1. Docker 환경에서 실행

```bash
# 최신 이미지 pull
docker pull example/erp:latest

# 기본 실행
docker run --rm example/erp:latest --help

# 데이터 볼륨과 함께 실행
docker run --rm -v $(pwd)/data:/app/data example/erp:latest
```

#### 2. Docker Compose 사용

```yaml
# docker-compose.yml
version: "3.8"

services:
  erp:
    image: example/erp:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/erp
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
    depends_on:
      - db

  db:
    image: postgres:13-alpine
    environment:
      - POSTGRES_DB=erp
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

```bash
# 실행
docker-compose up -d

# CLI 명령 실행
docker-compose exec erp erp --help
```

### 패키지 매니저 설치

#### Homebrew (macOS/Linux)

```bash
# Tap 추가
brew tap example/erp-cli

# 설치
brew install erp-cli

# 업데이트
brew upgrade erp-cli
```

#### Chocolatey (Windows)

```powershell
# 설치
choco install erp-cli

# 업데이트
choco upgrade erp-cli
```

#### APT (Ubuntu/Debian)

```bash
# GPG 키 추가
curl -fsSL https://example.com/gpg | sudo apt-key add -

# 저장소 추가
echo "deb https://example.com/apt stable main" | sudo tee /etc/apt/sources.list.d/erp-cli.list

# 설치
sudo apt update
sudo apt install erp-cli
```

#### YUM/DNF (CentOS/RHEL/Fedora)

```bash
# 저장소 추가
sudo tee /etc/yum.repos.d/erp-cli.repo << EOF
[erp-cli]
name=ERP CLI Repository
baseurl=https://example.com/rpm
enabled=1
gpgcheck=1
gpgkey=https://example.com/gpg
EOF

# 설치
sudo yum install erp-cli  # 또는 sudo dnf install erp-cli
```

## 데이터베이스 설정

### SQLite (개발/소규모 운영)

```bash
# 자동으로 설정됨 - 추가 작업 불필요
erp migrate init
```

### PostgreSQL (권장 - 프로덕션)

#### 1. PostgreSQL 설치

**Ubuntu/Debian:**

```bash
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**CentOS/RHEL:**

```bash
sudo yum install postgresql-server postgresql-contrib
sudo postgresql-setup initdb
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**macOS:**

```bash
brew install postgresql
brew services start postgresql
```

**Windows:**
[PostgreSQL 공식 웹사이트](https://www.postgresql.org/download/windows/)에서 설치 프로그램 다운로드

#### 2. 데이터베이스 및 사용자 생성

```bash
# PostgreSQL에 접속
sudo -u postgres psql

# 데이터베이스 및 사용자 생성
CREATE DATABASE erp_db;
CREATE USER erp_user WITH PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE erp_db TO erp_user;
\q
```

#### 3. 연결 설정

```bash
# 환경 변수 설정
export DATABASE_URL="postgresql://erp_user:secure_password@localhost:5432/erp_db"

# 또는 설정 파일에 추가
echo 'DATABASE_URL="postgresql://erp_user:secure_password@localhost:5432/erp_db"' >> ~/.env
```

### Redis (선택사항 - 캐싱)

#### 설치

**Ubuntu/Debian:**

```bash
sudo apt install redis-server
sudo systemctl start redis-server
sudo systemctl enable redis-server
```

**macOS:**

```bash
brew install redis
brew services start redis
```

**Windows:**

```bash
# Windows Subsystem for Linux 사용 권장
# 또는 Redis for Windows: https://github.com/microsoftarchive/redis/releases
```

#### 설정

```bash
# 환경 변수 설정
export REDIS_URL="redis://localhost:6379"
```

## 초기 설정

### 1. 데이터베이스 초기화

```bash
# 데이터베이스 마이그레이션 실행
erp migrate init

# 상태 확인
erp migrate status
```

### 2. 기본 설정

```bash
# 설정 파일 위치 확인
erp config path

# 기본 설정 조회
erp config list

# 필수 설정 값들
erp config set currency "KRW"
erp config set timezone "Asia/Seoul"
erp config set company.name "우리 회사"
```

### 3. 첫 번째 사용자 생성 (향후 구현)

```bash
# 관리자 계정 생성
erp users create-admin --username "admin" --email "admin@company.com"
```

### 4. 샘플 데이터 추가 (선택사항)

```bash
# 샘플 제품 추가
erp inventory add "노트북" --sku "LAPTOP001" --quantity 10 --price 1200000 --category "전자제품"
erp inventory add "마우스" --sku "MOUSE001" --quantity 50 --price 25000 --category "전자제품"

# 샘플 고객 추가
erp customers add "김철수" --email "kim@example.com" --phone "010-1234-5678"

# 샘플 주문 생성
erp sales create-order --customer-id 1 --product-sku "LAPTOP001" --quantity 1
```

## 설정 파일

### 환경 변수

ERP CLI는 다음 환경 변수를 지원합니다:

```bash
# 필수 설정
export DATABASE_URL="postgresql://user:pass@host:port/database"

# 선택적 설정
export REDIS_URL="redis://localhost:6379"
export JWT_SECRET="your-secure-jwt-secret"
export RUST_LOG="info"  # trace, debug, info, warn, error
export ERP_CONFIG_PATH="/path/to/config"
export ERP_ENV="production"  # development, staging, production
```

### 설정 파일 위치

**Linux/macOS:**

- 전역 설정: `/etc/erp/config.toml`
- 사용자 설정: `~/.config/erp/config.toml`
- 프로젝트 설정: `./config/config.toml`

**Windows:**

- 전역 설정: `C:\ProgramData\ERP\config.toml`
- 사용자 설정: `%APPDATA%\ERP\config.toml`
- 프로젝트 설정: `.\config\config.toml`

### 설정 파일 예시

```toml
# config/production.toml
[database]
url = "postgresql://erp_user:password@localhost:5432/erp_db"
max_connections = 20
min_connections = 5

[server]
host = "0.0.0.0"
port = 8080

[auth]
jwt_secret = "your-production-jwt-secret"
jwt_expires_in = 3600

[logging]
level = "info"
format = "json"

[company]
name = "우리 회사"
currency = "KRW"
timezone = "Asia/Seoul"

[features]
enable_cache = true
enable_audit = true
```

## 업그레이드

### 바이너리 업그레이드

```bash
# 현재 버전 확인
erp --version

# 새 버전 다운로드 및 교체
curl -L https://github.com/example/erp-cli/releases/latest/download/erp-linux-amd64 -o /tmp/erp
sudo mv /tmp/erp /usr/local/bin/erp
chmod +x /usr/local/bin/erp

# 데이터베이스 마이그레이션 (필요시)
erp migrate up
```

### Docker 업그레이드

```bash
# 새 이미지 pull
docker pull example/erp:latest

# 컨테이너 재시작
docker-compose down
docker-compose up -d
```

### 패키지 매니저 업그레이드

```bash
# Homebrew
brew upgrade erp-cli

# APT
sudo apt update && sudo apt upgrade erp-cli

# Chocolatey
choco upgrade erp-cli
```

## 문제 해결

### 일반적인 문제

#### 1. 명령어를 찾을 수 없음

**증상:** `command not found: erp` 또는 `'erp'은(는) 내부 또는 외부 명령...`

**해결방법:**

```bash
# PATH 확인
echo $PATH  # Linux/macOS
echo %PATH%  # Windows

# 바이너리 위치 확인
which erp  # Linux/macOS
where erp  # Windows

# PATH에 추가
export PATH="/usr/local/bin:$PATH"  # Linux/macOS
```

#### 2. 데이터베이스 연결 오류

**증상:** `Database connection failed`

**해결방법:**

```bash
# 연결 문자열 확인
erp config get database.url

# 데이터베이스 서비스 상태 확인
sudo systemctl status postgresql  # Linux
brew services list | grep postgres  # macOS

# 연결 테스트
erp migrate test
```

#### 3. 권한 오류

**증상:** `Permission denied`

**해결방법:**

```bash
# 실행 권한 추가
chmod +x /usr/local/bin/erp

# 소유권 확인
ls -la /usr/local/bin/erp

# 관리자 권한으로 설치
sudo cp erp /usr/local/bin/
```

#### 4. 포트 충돌 (서버 모드)

**증상:** `Port 8080 is already in use`

**해결방법:**

```bash
# 사용 중인 포트 확인
netstat -tulpn | grep 8080  # Linux
lsof -i :8080  # macOS
netstat -ano | findstr :8080  # Windows

# 다른 포트 사용
erp server start --port 8081
```

### 로그 확인

#### 로그 파일 위치

**Linux/macOS:**

- `/var/log/erp/erp.log` (시스템 서비스)
- `~/.local/share/erp/logs/erp.log` (사용자 실행)

**Windows:**

- `C:\ProgramData\ERP\logs\erp.log` (시스템 서비스)
- `%APPDATA%\ERP\logs\erp.log` (사용자 실행)

#### 로그 레벨 조정

```bash
# 디버그 모드로 실행
RUST_LOG=debug erp inventory list

# 특정 모듈만 디버그
RUST_LOG=erp_cli::modules::inventory=debug erp inventory list

# 실시간 로그 모니터링
tail -f ~/.local/share/erp/logs/erp.log
```

### 성능 최적화

#### 데이터베이스 최적화

```bash
# 데이터베이스 인덱스 최적화
erp maintenance optimize-indexes

# 통계 업데이트
erp maintenance update-stats

# 연결 풀 설정 조정
erp config set database.max_connections 30
```

#### 캐시 설정

```bash
# Redis 캐시 활성화
erp config set cache.enabled true
erp config set cache.ttl 3600

# 캐시 상태 확인
erp cache status
```

## 지원 및 문의

### 자주 묻는 질문 (FAQ)

**Q: 여러 데이터베이스를 동시에 사용할 수 있나요?**
A: 현재는 하나의 주 데이터베이스만 지원합니다. 향후 버전에서 멀티 데이터베이스 지원을 계획하고 있습니다.

**Q: 기존 데이터를 다른 시스템에서 가져올 수 있나요?**
A: CSV 가져오기 기능을 제공합니다. `erp import` 명령어를 참조하세요.

**Q: 백업은 어떻게 하나요?**
A: `erp backup create` 명령어로 백업할 수 있습니다. 자세한 내용은 사용자 가이드를 참조하세요.

**Q: 클러스터 환경에서 실행할 수 있나요?**
A: 현재는 단일 인스턴스만 지원합니다. 고가용성 클러스터 지원은 로드맵에 포함되어 있습니다.

---

설치에 대한 추가 질문이나 문제가 있으시면 언제든지 문의해 주세요!
