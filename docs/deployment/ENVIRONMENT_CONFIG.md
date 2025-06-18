# HAL9 환경 설정 가이드

## 📋 개요

HAL9는 환경 변수를 통해 설정을 관리합니다. 이 가이드는 개발, 스테이징, 프로덕션 환경에서의 설정 방법을 설명합니다.

## 🔧 빠른 시작

1. **환경 파일 복사**
   ```bash
   cp .env.example .env
   ```

2. **시크릿 생성**
   ```bash
   ./scripts/generate-secrets.sh
   ```

3. **환경 변수 설정**
   ```bash
   # 필수 변수만 설정
   export JWT_SECRET="your-secret-key"
   export DATABASE_URL="postgresql://user:pass@localhost/hal9"
   ```

## 📁 환경 파일 구조

```
.env.example        # 예제 설정 (커밋됨)
.env               # 로컬 개발 설정 (무시됨)
.env.production    # 프로덕션 템플릿 (무시됨)
.env.staging       # 스테이징 설정 (무시됨)
```

## 🔐 시크릿 관리

### 개발 환경
```bash
# .env 파일 사용
JWT_SECRET=dev-secret-key
DATABASE_URL=postgresql://localhost/hal9_dev
```

### 프로덕션 환경

**절대 하지 말아야 할 것:**
- ❌ 시크릿을 코드에 하드코딩
- ❌ 시크릿을 Git에 커밋
- ❌ 평문으로 시크릿 저장

**권장 방법:**

1. **AWS Secrets Manager**
   ```bash
   aws secretsmanager create-secret \
     --name hal9/production/jwt-secret \
     --secret-string "$(openssl rand -base64 32)"
   ```

2. **HashiCorp Vault**
   ```bash
   vault kv put secret/hal9/production \
     jwt_secret="$(openssl rand -base64 32)" \
     db_password="$(openssl rand -base64 16)"
   ```

3. **Kubernetes Secrets**
   ```yaml
   apiVersion: v1
   kind: Secret
   metadata:
     name: hal9-secrets
   type: Opaque
   data:
     jwt-secret: <base64-encoded-secret>
   ```

## 🚀 환경별 설정

### 개발 환경
```bash
ENVIRONMENT=development
LOG_LEVEL=debug
CLAUDE_MODE=mock
DEV_MODE=true
```

### 스테이징 환경
```bash
ENVIRONMENT=staging
LOG_LEVEL=info
CLAUDE_MODE=real
RATE_LIMIT_PER_MINUTE=60
```

### 프로덕션 환경
```bash
ENVIRONMENT=production
LOG_LEVEL=warn
CLAUDE_MODE=real
RATE_LIMIT_PER_MINUTE=30
SSL_ENABLED=true
ENABLE_CLUSTERING=true
```

## 📊 주요 설정 카테고리

### 1. 서버 설정
| 변수 | 설명 | 기본값 | 프로덕션 권장 |
|------|------|--------|---------------|
| PORT | 서버 포트 | 3456 | 3456 |
| HOST | 바인딩 주소 | 0.0.0.0 | 0.0.0.0 |
| ENVIRONMENT | 실행 환경 | development | production |

### 2. 보안 설정
| 변수 | 설명 | 기본값 | 프로덕션 권장 |
|------|------|--------|---------------|
| JWT_SECRET | JWT 서명 키 | dev-secret | 강력한 랜덤 키 |
| SESSION_SECRET | 세션 암호화 키 | dev-session | 강력한 랜덤 키 |
| ALLOWED_ORIGINS | CORS 허용 도메인 | localhost | 실제 도메인만 |

### 3. 데이터베이스 설정
| 변수 | 설명 | 기본값 | 프로덕션 권장 |
|------|------|--------|---------------|
| DATABASE_URL | PostgreSQL 연결 문자열 | localhost | SSL 연결 사용 |
| DATABASE_POOL_SIZE | 연결 풀 크기 | 10 | 50-100 |
| DATABASE_SSL_MODE | SSL 모드 | - | require |

### 4. 성능 설정
| 변수 | 설명 | 기본값 | 프로덕션 권장 |
|------|------|--------|---------------|
| WORKER_THREADS | 워커 스레드 수 | 4 | 0 (CPU 수) |
| NEURON_POOL_SIZE | 뉴런 풀 크기 | 10000 | 100000 |
| CACHE_TTL_SECONDS | 캐시 TTL | 300 | 3600 |

## 🔍 설정 검증

### 설정 확인 스크립트
```bash
# 필수 환경 변수 확인
./scripts/check-env.sh

# 설정 덤프 (민감 정보 마스킹)
cargo run --bin hal9-config-dump
```

### 런타임 검증
```rust
// 앱 시작 시 자동 검증
let config = Config::from_env()?;
config.validate()?;
```

## 🐳 Docker 환경

### docker-compose.yml
```yaml
services:
  hal9:
    env_file:
      - .env
    environment:
      - ENVIRONMENT=production
      - DATABASE_URL=${DATABASE_URL}
```

### Dockerfile
```dockerfile
# 빌드 시 ARG 사용
ARG ENVIRONMENT=production
ENV ENVIRONMENT=${ENVIRONMENT}

# 런타임 시 환경 변수 주입
CMD ["hal9-server"]
```

## 🌐 클라우드 배포

### AWS ECS
```json
{
  "family": "hal9-task",
  "taskRoleArn": "arn:aws:iam::xxx:role/hal9-task-role",
  "secrets": [
    {
      "name": "JWT_SECRET",
      "valueFrom": "arn:aws:secretsmanager:region:xxx:secret:hal9/jwt-secret"
    }
  ]
}
```

### Google Cloud Run
```bash
gcloud run deploy hal9 \
  --set-env-vars="ENVIRONMENT=production" \
  --set-secrets="JWT_SECRET=hal9-jwt-secret:latest"
```

### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
      - name: hal9
        envFrom:
        - configMapRef:
            name: hal9-config
        - secretRef:
            name: hal9-secrets
```

## 🔒 보안 체크리스트

- [ ] 프로덕션에서 기본 시크릿 사용하지 않음
- [ ] 환경 파일이 .gitignore에 포함됨
- [ ] 시크릿이 로그에 노출되지 않음
- [ ] HTTPS/TLS 활성화됨
- [ ] 정기적인 시크릿 로테이션 설정됨

## 🚨 문제 해결

### 환경 변수가 로드되지 않음
```bash
# .env 파일 확인
ls -la .env

# 환경 변수 직접 설정
export $(cat .env | xargs)
```

### 시크릿 관련 오류
```bash
# 시크릿 길이 확인 (최소 32바이트)
echo -n "$JWT_SECRET" | wc -c

# Base64 인코딩 확인
echo "$JWT_SECRET" | base64 -d
```

### 설정 디버깅
```bash
# 환경 변수 출력 (민감 정보 주의!)
env | grep HAL9_

# 설정 검증 모드로 실행
HAL9_CONFIG_VALIDATE_ONLY=true cargo run
```

## 📚 추가 리소스

- [12 Factor App - Config](https://12factor.net/config)
- [OWASP Configuration Management](https://owasp.org/www-project-proactive-controls/)
- [AWS Secrets Manager Best Practices](https://docs.aws.amazon.com/secretsmanager/latest/userguide/best-practices.html)