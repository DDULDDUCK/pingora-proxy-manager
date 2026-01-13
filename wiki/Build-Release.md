# Build & Release

How we build and release Pingora Proxy Manager.

## Multi-stage Docker Build
We use a single `Dockerfile` with multiple stages to produce a slim production image.

### Stage 1: Frontend Build
- Uses `node:20-alpine`.
- Installs dependencies and runs `npm run build`.
- Outputs static files to `frontend/dist`.

### Stage 2: Backend Build
- Uses `rust:1.81-slim`.
- Installs system dependencies (OpenSSL, pkg-config).
- Compiles the Rust binary in release mode.

### Stage 3: Final Image
- Uses a slim Debian or Alpine base.
- Copies the compiled binary from Stage 2.
- Copies the static frontend files from Stage 1 into the `static/` directory relative to the binary.
- Sets up the entrypoint.

## Automated Releases
We use GitHub Actions to automate the build and push process:
1. **On Tag**: When a new version tag (e.g., `v1.2.3`) is pushed, a workflow is triggered.
2. **Build**: The Docker image is built for `amd64` and `arm64` architectures.
3. **Push**: The images are pushed to Docker Hub with the version tag and the `latest` tag.

## Local Build
To build the image locally:
```bash
docker build -t pingora-proxy-manager:local .
```

## Versioning
We follow [Semantic Versioning (SemVer)](https://semver.org/):
- `MAJOR`: Incompatible API changes.
- `MINOR`: Functionality added in a backwards-compatible manner.
- `PATCH`: Backwards-compatible bug fixes.
