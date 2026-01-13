# Contributing Guide

First off, thank you for considering contributing to Pingora Proxy Manager! It's people like you that make it a great tool for everyone.

## Code of Conduct
Please be respectful and professional in all interactions.

## How Can I Contribute?

### Reporting Bugs
- Check the [Issues](https://github.com/dduldduck/pingora-proxy-manager/issues) page to see if the bug has already been reported.
- If not, create a new issue. Include as much detail as possible: steps to reproduce, expected behavior, and actual behavior.

### Suggesting Enhancements
- Open an issue and label it as an `enhancement`.
- Describe the feature you'd like to see and why it would be useful.

### Pull Requests
1. Fork the repository.
2. Create a new branch for your feature or bugfix (`git checkout -b feature/awesome-feature`).
3. Make your changes.
4. Run tests (`cargo test` for backend).
5. Commit your changes (`git commit -m 'Add some awesome feature'`).
6. Push to the branch (`git push origin feature/awesome-feature`).
7. Open a Pull Request.

## Coding Standards

### Backend (Rust)
- Follow standard Rust naming conventions (`snake_case` for variables/functions, `PascalCase` for types).
- Use `cargo fmt` to format your code.
- Write doc comments for public functions.
- Keep functions small and focused.

### Frontend (React)
- Use functional components and hooks.
- Use Tailwind CSS for styling.
- Follow the existing component structure in `frontend/src/components`.

## Build & CI
- Every PR will trigger a GitHub Action to build the backend and frontend.
- Ensure the build passes before asking for a review.

---
Thank you for your contribution!
