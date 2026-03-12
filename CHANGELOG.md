# Changelog

## [0.1.9](https://github.com/FINGU-GRINDA/claude-rinda-plugin/compare/rinda-plugin-v0.1.8...rinda-plugin-v0.1.9) (2026-03-12)


### Bug Fixes

* **ci:** use cross for aarch64-linux-musl to fix glibc link errors ([#128](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/128)) ([c625f73](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/c625f73b19e4bdc4e2a787d5cf8eacd760834875))
* **mcp,cli:** cargo fmt and clippy fixes for [#130](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/130) ([#131](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/131)) ([ac8d01a](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/ac8d01ab20bb84dd9354107926d89da7221bf729))
* **mcp:** resolve workspace ID and auto-refresh RINDA JWT ([#130](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/130)) ([fc99ac1](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/fc99ac11973bf19b7cffc8e13f6ff39723224961))

## [0.1.8](https://github.com/FINGU-GRINDA/claude-rinda-plugin/compare/rinda-plugin-v0.1.7...rinda-plugin-v0.1.8) (2026-03-11)


### Features

* **cli:** expose workspace list subcommand ([#117](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/117)) ([b7315e2](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/b7315e2215b6b59be59717daac848dfc222808a9))
* **mcp,cli:** add customer group management (CRUD + members) ([#120](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/120)) ([d0f24b8](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/d0f24b80ff608d045a8c90134353015cc02a0064))
* **mcp,cli:** add lead/buyer management CRUD, search, status ([#121](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/121)) ([4ddb5d3](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/4ddb5d3204441d68b9e6052d529b895701cdd36f))
* **mcp,cli:** add search session history (list past searches) ([#119](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/119)) ([7f6196b](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/7f6196b3722b7f6d20a60fc457c1d1713a8d831b))
* **mcp:** add buyer_messages and order_history tools, update READMEs ([#114](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/114)) ([fdd84b4](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/fdd84b49c843c80f4642136c093e7f33935a23aa))
* **mcp:** expose rinda_workspace_list tool ([#118](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/118)) ([398cae6](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/398cae6f245bd35ffc3e9163539149f281df8d6e))


### Bug Fixes

* **ci:** switch Linux builds to musl for GLIBC-free static binaries ([#123](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/123)) ([8c54e26](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/8c54e26f3c7aad61049751369743dc8c1dd77769))
* **mcp,cli:** forward steps in sequence create, fail-fast on empty workspace, verify creation ([#126](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/126)) ([56d0e4c](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/56d0e4c45d24f2a2cc2a76f9d92b27b28efdd5e0))
* **mcp,cli:** use SSE streaming for buyer search endpoint ([#125](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/125)) ([e9477e9](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/e9477e9d25f5f18cdec474234a30fe9495e4aaeb))
* **mcp:** handle RINDA refresh token in OAuth callback ([#109](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/109)) ([c066fc4](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/c066fc460d036fd9e5756692239b90943a2c83ad))

## [0.1.7](https://github.com/FINGU-GRINDA/claude-rinda-plugin/compare/rinda-plugin-v0.1.6...rinda-plugin-v0.1.7) (2026-03-11)


### Features

* add standalone bin/install.sh for MCP server ([#79](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/79)) ([#93](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/93)) ([a9dc65b](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/a9dc65b67bce38bd55d63689a44fbb1fc7f087e1))
* **ci:** build and distribute MCP server binary ([#91](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/91)) ([b90f5a7](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/b90f5a7c8299bba77ac87551ae0efdc77565c55b))
* **mcp:** adapt OAuth flow to use /cli-auth callback redirect ([#108](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/108)) ([138cbb8](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/138cbb873f013e5cd75b6bf685f850e2e7b16b22))
* **mcp:** convert MCP server from stdio to remote HTTP transport ([#97](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/97)) ([e572564](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/e5725644a2983bcd40fd4ad671725b70f9977200))
* **mcp:** Dockerfile and Dokploy deployment config ([#100](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/100)) ([1512f54](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/1512f5440e6627aa7bd539c0021ca8efe46700d4))
* **mcp:** implement all 15 MCP tools ([#89](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/89)) ([a9aee9d](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/a9aee9d0a7b0d63363303d3475f67bf9973d771f))
* **mcp:** implement OAuth 2.0 authentication flow ([#99](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/99)) ([ad408ad](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/ad408adc51936005e564d5f6dcf576c6a87e5c22))
* **mcp:** refactor auth from local credentials to Bearer token passthrough ([#98](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/98)) ([abee769](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/abee769d33e9fb7cf781457dfc44a8b21c696d6e))
* **mcp:** scaffold MCP server crate ([#88](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/88)) ([349f4f4](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/349f4f42dead834a67762b83d5bf00d5c1e53f8c))
* **plugin:** register MCP server in plugin.json ([#90](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/90)) ([b482cb9](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/b482cb92ac6cbfe59d9388ca953524d0e60cf3b8))


### Bug Fixes

* **mcp:** add RFC 9728 protected resource metadata for OAuth discovery ([#105](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/105)) ([7aebd91](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/7aebd91ede1a880e4fe41a553e799c7ed053aa5b))
* **mcp:** remove bare OAuth URL from auth_login response ([#103](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/103)) ([325ddf5](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/325ddf5d182a371b70609240e199f5ea17c98495))
* **mcp:** remove redundant rinda_auth_login tool ([#106](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/106)) ([6513eb4](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/6513eb40e33610a92de0ea5ae4554ef2f2538f51))
* **mcp:** replace broken /cli-auth with OAuth authorize endpoint ([#101](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/101)) ([58a905b](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/58a905b693503a20b07407d0e983e2c144440a01))
* **mcp:** serve MCP endpoint at root path instead of /mcp ([#104](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/104)) ([e04e465](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/e04e4659426518383a04f92eba9439d35434f808))


### Refactoring

* extract shared crate (crates/common) ([#86](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/86)) ([3b34e3b](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/3b34e3bbb0b890fc65ec7920ecbb08563fdc8d7e))


### Documentation

* update install.sh references and docs for MCP server ([#85](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/85)) ([#92](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/92)) ([a727265](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/a72726548df92c15a6ac39edb43deeeafc146fdd))

## [0.1.6](https://github.com/FINGU-GRINDA/claude-rinda-plugin/compare/rinda-plugin-v0.1.5...rinda-plugin-v0.1.6) (2026-03-10)


### Features

* **ci:** bundle rinda-ai plugin folder as zip in GitHub Releases ([#78](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/78)) ([4acab35](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/4acab358963faace9e7589fb7b4fadf641c6bafd))
* **cli:** add buyer clarification response flow ([#75](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/75)) ([6208b49](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/6208b495115e260ee86e764d5ab2039999121220))

## [0.1.5](https://github.com/FINGU-GRINDA/claude-rinda-plugin/compare/rinda-plugin-v0.1.4...rinda-plugin-v0.1.5) (2026-03-10)


### Bug Fixes

* install script reads version from plugin.json ([#70](https://github.com/FINGU-GRINDA/claude-rinda-plugin/issues/70)) ([e28a8ee](https://github.com/FINGU-GRINDA/claude-rinda-plugin/commit/e28a8ee85ec3675e1baa2d972baa013ba09902f5))

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
