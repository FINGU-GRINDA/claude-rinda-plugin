# Changelog

## [0.1.4](https://github.com/FINGU-GRINDA/claude-rinda-plugin/compare/rinda-plugin-v0.1.3...rinda-plugin-v0.1.4) (2026-03-10)


### Features

* **ci:** add daily OpenAPI spec sync workflow ([#34](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/34)) ([a318d1e](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/a318d1ea3c1d19ada819b77ef75bbc7be12b27a9))
* **ci:** automate releases with release-please ([#38](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/38)) ([b40508e](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/b40508ea3f240431ca880b8bd8098770ca512d01))
* **cli:** add install script and document API status ([#50](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/50)) ([7ddcd65](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/7ddcd65a797f534e29dc44b96ce89aaf59fb7dab))
* **cli:** build OAuth login flow for rinda-cli ([#3](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/3)) ([#22](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/22)) ([3e2fa44](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/3e2fa4457d8e67bf77e38b72f6046360bfaa1272))
* **cli:** implement CLI commands for all RINDA plugin flows ([#41](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/41)) ([9a1787e](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/9a1787eb25cdab6b938e8edb884d619f5461fb39))
* **cli:** implement token refresh and ensure-valid ([#4](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/4)) ([9d7f45f](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/9d7f45f3f6bbffa5d0fb6e340273f0ea5075f1bc))
* **commands:** add 6 plugin command markdown files ([#16](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/16)) ([23b1bbb](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/23b1bbb22ab44df33bb1ab4e0407e947f6739c43))
* initialize Rust workspace and crates structure ([#11](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/11)) ([ce20a4e](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/ce20a4e9ee9bfcca62622389ed3c336466bac655))
* **plugin:** add plugin manifest and directory structure ([#13](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/13)) ([12a5401](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/12a5401f3ed87d8864f899dd96c59fe772e9bca7))
* **plugin:** create RINDA orchestrator sub-agent ([#31](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/31)) ([ca8758d](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/ca8758df7b4f1950e88bae123c3b64ca6aee9d3e))
* **plugin:** restructure to match official Claude Code conventions ([#30](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/30)) ([51cc6cc](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/51cc6cc9b8488b9124f599b361a79de4c6425369))
* **sdk:** add SDK tests and API flows doc ([#39](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/39)) ([e59e2b7](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/e59e2b70eacbcdce1f1241525b8a8848544a690f))
* **sdk:** build rinda-sdk crate with full RINDA API coverage ([#17](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/17)) ([5364d36](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/5364d363c3696ed611b16ace03b959b8647bd17f))
* **sdk:** regenerate from OpenAPI spec via openapi-generator ([#26](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/26)) ([8807f7c](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/8807f7cb6b9f4e94afd018ba61243360a764ef2e))
* write enrichment sub-agent ([#8](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/8)) ([#14](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/14)) ([6ee8d68](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/6ee8d68b9e875f2cfc029c6c3702192277abed0b))
* write plugin skills (3 domain knowledge files) ([#15](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/15)) ([ee23237](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/ee23237d55d61091726156633f59c342011dd4ca))


### Bug Fixes

* **ci:** build and attach CLI binaries to release-please releases ([#47](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/47)) ([e3336d9](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/e3336d9c38484de050129f698cddc06c0b8e1419))
* **ci:** point release-please to crate path instead of workspace root ([#45](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/45)) ([2e33f1f](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/2e33f1f6c41c33e9ed9e7ff7df954b37e6585834))
* **ci:** resolve release-please workspace version parsing ([#44](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/44)) ([815d705](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/815d705bfb7118dcc0ec7dbddbec0f54b377ddad))
* **ci:** switch release-please to root-level simple release type ([#66](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/66)) ([fe4e1f2](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/fe4e1f253d7830478957f2307f51d5b6dac6dcd7))
* **ci:** use component-scoped release-please output keys ([#51](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/51)) ([9e677b6](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/9e677b69c3693c9f262c3941c9d432e8d7520ca8))
* **ci:** use generic updater for Cargo.toml in release-please ([#67](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/67)) ([85f2b1f](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/85f2b1f72732c9ea7e8bfb9f4842850f82d9898a))
* **ci:** use PR instead of direct push for openapi sync ([#65](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/65)) ([5917926](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/59179267dcb20d8974515acf6fe5eaef9166028f))
* **cli:** fix duplicate variable in ensure-valid error handling ([#53](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/53)) ([b9b8f7b](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/b9b8f7b55ebcb44cd20b65c45ca2392a0f003499))
* **cli:** track extra-files in release-please config ([#52](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/52)) ([03b41a2](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/03b41a23bb5c5e49e04b2c33b4f753a450a49c93))
* **cli:** unwrap API data envelope in auth and refresh flows ([#48](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/48)) ([d4642bf](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/d4642bf6dc8d2b245f7fb6588010c931c5ed2901))
* **plugin:** add YAML frontmatter to skills and agent files ([#23](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/23)) ([a20d81a](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/a20d81aad43cede84b77d81a3958d2fd2dc61242))
* replace broken version workflow with release-please ([#63](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/63)) ([a81801b](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/a81801b4be8357ad39247b4909a66381927a33f0))


### Performance

* **ci:** build only linux in CI, full matrix in release ([#69](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/69)) ([d71f61c](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/d71f61c7267eb0deac810d3d1469f71cdaa312ee))
* reduce release binary size by 48% ([#64](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/64)) ([a210815](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/a210815f67d1d78adca265e4badea20ca0c200ee))


### Refactoring

* **cli:** rework auth to use refresh token flow ([#42](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/42)) ([6bf362c](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/6bf362ce9b5e162689ad7aa1d974c85a2cf160e4))
* replace rinda-agent with CLI-first skill ([#62](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/62)) ([3d5cffe](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/3d5cffe3aa0b9a7ffa90f4d3f4cf37378a44a359))
* restructure repo as plugin marketplace ([#61](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/61)) ([ff2a6a6](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/ff2a6a63e24542985af2851fe504e4ecda26aac3))
* **sdk:** generate SDK directly from openapi.json ([#43](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/43)) ([a910907](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/a9109072aa83cb8b8385215e8c2d0aa369801a9f))
* **sdk:** restructure workspace with progenitor SDK generation ([#36](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/36)) ([de5022c](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/de5022c37d3f3fc0d75ffc6f2e8631e5bfe05974))


### Documentation

* add LICENSE and CHANGELOG.md for marketplace distribution ([#21](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/21)) ([#24](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/24)) ([cbed214](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/cbed21411993ca72fe231ac5494ad7cd03a3245b))
* **agent:** add CLI setup and first-time login instructions ([#55](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/55)) ([91c8fb9](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/91c8fb953508254122b804f37d3db195d0858e8a))
* **commands:** update connect command for token-based auth ([#56](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/56)) ([a9f67f0](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/a9f67f0705738033b4f706616635184f737818a9))
