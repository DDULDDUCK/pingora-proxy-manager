# === Stage 1: Frontend Build ===
FROM node:20-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
# 백엔드가 같은 도메인(포트 81)에서 서빙하므로 API URL을 상대 경로(/api)로 수정하거나
# 빌드 시점에 환경변수를 주입해야 함. 여기서는 코드상 localhost:81로 되어있으므로 주의.
# App.tsx의 API_BASE를 수정하는 것이 좋음. (아래 sed 명령어로 핫픽스)
RUN sed -i 's|/api|/api|g' src/App.tsx
RUN npm run build

# === Stage 2: Backend Build ===
# GLIBC 버전 호환성을 위해 bookworm 기반 이미지 사용
FROM rust:slim-bookworm AS backend-builder
WORKDIR /app

# 필수 빌드 도구 설치 (cmake, clang, perl 등)
RUN apt-get update && apt-get install -y cmake clang pkg-config libssl-dev perl && rm -rf /var/lib/apt/lists/*

# 워크스페이스 설정 파일 복사
COPY Cargo.toml Cargo.lock ./

# 백엔드 패키지 파일 복사
COPY backend/Cargo.toml ./backend/Cargo.toml

# 더미 소스로 의존성만 먼저 빌드
RUN mkdir -p backend/src && echo "fn main() {}" > backend/src/main.rs

# 워크스페이스 멤버 빌드
RUN cargo build --release --bin backend

# 실제 소스 복사 후 다시 빌드
COPY backend/src ./backend/src
# touch로 mtime 갱신하여 재빌드 유도
RUN touch backend/src/main.rs
RUN cargo build --release --bin backend

# === Stage 3: Final Runtime ===
FROM debian:bookworm-slim
WORKDIR /app

# 필수 패키지 설치 (SSL 인증서, Certbot 및 DNS 플러그인)
# python3-certbot-dns-cloudflare: Cloudflare DNS용 플러그인
# 필요에 따라 python3-certbot-dns-route53 (AWS), python3-certbot-dns-google 등 추가 가능
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    certbot \
    python3-certbot-dns-cloudflare \
    python3-certbot-dns-route53 \
    python3-certbot-dns-digitalocean \
    python3-certbot-dns-google \
    && rm -rf /var/lib/apt/lists/*

# 실행 파일 복사
COPY --from=backend-builder /app/target/release/backend /app/pingora-pm

# 프론트엔드 빌드 결과물 복사
COPY --from=frontend-builder /app/frontend/dist /app/static

# 데이터 저장소 및 Certbot 디렉토리 생성
RUN mkdir -p /app/data /etc/letsencrypt

# 포트 노출 (8080: 프록시, 81: 관리자 UI)
EXPOSE 8080 81

# 실행
CMD ["/app/pingora-pm"]
