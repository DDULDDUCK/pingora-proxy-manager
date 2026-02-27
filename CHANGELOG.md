# Changelog

## [1.2.1](https://github.com/DDULDDUCK/pingora-proxy-manager/compare/v1.2.0...v1.2.1) (2026-02-27)


### Bug Fixes

* **acme:** use explicit dns-hetzner authenticator ([bcfc11a](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/bcfc11a581b301c474c178b317ea5f61fdfd5daa))

## [1.2.0](https://github.com/DDULDDUCK/pingora-proxy-manager/compare/v1.1.0...v1.2.0) (2026-02-04)


### Features

* add AddHostDialog and EditHostDialog components for managing hosts ([31b5b83](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/31b5b83c0710ad91dd35bf6e75fdb9cbf5d2c8ee))
* add Audit Logs and User Management features ([e9310c1](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/e9310c1cb63f83a5a4521f4053ad4c944176afcc))
* add Korean language support and enhance UI text ([1bd8e81](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/1bd8e813b54cfb25ebb036e2bb9f9c1739e76466))
* add multi-target support for hosts and locations with load balancing ([66f98f3](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/66f98f3274e446a8037ec01b0522ef33a5699ebe))
* add README.md with project overview, features, installation instructions, and contribution guidelines ([a2e3b42](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/a2e3b42c4f6f40277f244a440164bc7ca45dc1a5))
* add support for managing custom headers in hosts; implement CRUD operations and UI integration ([0398953](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/039895337c7dd845ebb6a9971c114ee2bdf9112f))
* add UI components and styles for badges, buttons, cards, dialogs, inputs, labels, selects, tables, and notifications ([e9c5156](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/e9c5156a60aee52c038157daebbf830eaa9cb9e0))
* add verify_ssl option to host and location configurations ([5522475](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/5522475a47cd123aa067a573dc98080f7edd68d4))
* **frontend:** improve dark mode theme and header UI ([48d82ef](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/48d82ef661eb708c2609f319fefbbbb8dd6feeb3))
* implement Access Lists management with CRUD operations ([b219928](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/b2199285cc2d4ff39e6af1cadb8bde70b7b09829))
* Implement API types and database interactions for user management, access lists, certificates, and DNS providers ([7aeebe6](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/7aeebe6f40d1b428536676408e9230cb14752b9e))
* Implement dynamic TLS certificate management and logging ([0299210](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/0299210c60b9ef65506526c05bd436e9fe5bc5e5))
* implement DynamicCertManager for SNI-based TLS certificate management ([239bf15](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/239bf153432be2021ab0b1c52fd0f6176addc11c))
* update version to 1.0.0 in Cargo.toml and enhance README with new features ([b84b8cf](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/b84b8cfa4407676fdf6ede34be88ab755e544277))
* update version to 1.0.1 in Cargo.toml, package.json, and package-lock.json; fix database URL in main.rs ([e41c657](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/e41c65783ea0a394deebd198c0dc8b80b731282a))
* update version to 1.0.2 in Cargo.toml, package.json, and package-lock.json; add IssueCertDialog component for SSL certificate requests ([444050e](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/444050eb057de219f4b0f75b4d85608a0a7e2d76))
* update version to 1.0.3 in Cargo.toml, package.json, and package-lock.json; add DNS provider support in AcmeManager and update provider templates in CertificatesTab ([2ce9261](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/2ce926117c13448f67885cff840b5e8c8a200243))


### Bug Fixes

* auto-create database directory if missing ([7189458](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/7189458c32b66e7153ebfe43bc9fcaf9817057e5))
* bump version to 1.0.8 and improve ACL and ACME filter logic ([d247025](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/d2470258a46748cefa67d39cfba978377ec4ec48))
* bump version to 1.0.8 in package-lock.json ([82c6e5f](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/82c6e5ff5e8ed85158f33dddf16cf54699cbd538))
* improve host parsing logic to prioritize URI host over manual Host header ([ceec468](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/ceec4688d58c81651e256e8666bc8d607df4d771))
* resolve data path issues and remove unused import ([aaeed12](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/aaeed12cb09cb9b372697843a15d91859dcc7777))
* update CI configuration to ignore master and main branches for push and pull_request events ([9b38f95](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/9b38f95fcc036cc3373dbe8da126da13733117c2))
* update formatTimestamp to use locale based on user settings ([18a7718](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/18a771848ec093e46132ee11de98530c1aa1fbcd))
* update repository clone URL in installation instructions ([cb8155d](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/cb8155d41ab6ac3acbbf6e0d30531fb627c0e12d))
* update repository clone URL in installation instructions ([5ffd235](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/5ffd23598424f577c5831003b4891a4d8900b068))
* update repository clone URL in installation instructions ([e5a9497](https://github.com/DDULDDUCK/pingora-proxy-manager/commit/e5a9497afaa39ce897355957e061f488cede2c5f))

## Changelog

All notable changes to this project will be documented in this file.
