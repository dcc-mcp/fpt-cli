# Changelog

## [0.2.26](https://github.com/dcc-mcp/fpt-cli/compare/v0.2.25...v0.2.26) (2026-09-07)


### Features

* add schedule.work-day-rules-read endpoint integration ([#135](https://github.com/dcc-mcp/fpt-cli/issues/135)) ([482ce1f](https://github.com/dcc-mcp/fpt-cli/commit/482ce1f90c6d671555d9eaf8f70540443f764315))
* add secure auth profiles ([76b8e88](https://github.com/dcc-mcp/fpt-cli/commit/76b8e887a7caa01000df0452d06b595eb75c3dea))
* integrate REST API version and OpenAPI spec endpoints ([#132](https://github.com/dcc-mcp/fpt-cli/issues/132)) ([f1903eb](https://github.com/dcc-mcp/fpt-cli/commit/f1903ebaad6ca3b6719e0d1ad01b9965b3a3ab32))
* integrate REST-based summarize and server-side batch API endpoints ([#137](https://github.com/dcc-mcp/fpt-cli/issues/137)) ([df323eb](https://github.com/dcc-mcp/fpt-cli/commit/df323eb39f9a1b4b053d29954c6a97f77ee1e3d0))
* migrate entity revive to REST and add token refresh support ([#139](https://github.com/dcc-mcp/fpt-cli/issues/139)) ([4511624](https://github.com/dcc-mcp/fpt-cli/commit/451162415ef838a7c6cb751d9e8579fa9e24bac0))


### Bug Fixes

* vendor secure profile backend ([cb60334](https://github.com/dcc-mcp/fpt-cli/commit/cb60334b0a10e7a0639affeef928e633bf82ea28))


### Miscellaneous Chores

* add #[must_use] annotations and improve type ergonomics ([#133](https://github.com/dcc-mcp/fpt-cli/issues/133)) ([f6303dd](https://github.com/dcc-mcp/fpt-cli/commit/f6303dd9ecde1565934bf0c13f823c8c0e05ab8b))
* apply clippy pedantic and nursery lint fixes ([#134](https://github.com/dcc-mcp/fpt-cli/issues/134)) ([e7507b4](https://github.com/dcc-mcp/fpt-cli/commit/e7507b4076c257b3430b7a1bed70f9cc163131c9))
* **deps:** update rust crate hyper to v1.9.0 ([#113](https://github.com/dcc-mcp/fpt-cli/issues/113)) ([5e2d339](https://github.com/dcc-mcp/fpt-cli/commit/5e2d339455ce4cebad2c5bf707627b5d17fa9273))
* **deps:** update rust crate semver to v1.0.28 ([#121](https://github.com/dcc-mcp/fpt-cli/issues/121)) ([98e9ff0](https://github.com/dcc-mcp/fpt-cli/commit/98e9ff04040617d68230c063ad15e8218cfae02a))
* fix clippy pedantic lint warnings and improve doc formatting ([#138](https://github.com/dcc-mcp/fpt-cli/issues/138)) ([973425d](https://github.com/dcc-mcp/fpt-cli/commit/973425d2acdf233b6969a9fae100d38ca80fb3fa))
* improve documentation and clean up minor code smells ([#136](https://github.com/dcc-mcp/fpt-cli/issues/136)) ([cb12ac1](https://github.com/dcc-mcp/fpt-cli/commit/cb12ac1634ee011780c35d31fb5ea298bcae1756))
* move fpt cli to dcc mcp ([9c2e7e4](https://github.com/dcc-mcp/fpt-cli/commit/9c2e7e4a3be368c8799c622bfa970f39811b90f3))

## [0.2.25](https://github.com/loonghao/fpt-cli/compare/v0.2.24...v0.2.25) (2026-04-02)


### Miscellaneous Chores

* **deps:** update rust crate zip to v8.5.0 ([ea92c7a](https://github.com/loonghao/fpt-cli/commit/ea92c7aa64b0ac66255ee91b923cecc38dff3ff3))

## [0.2.24](https://github.com/loonghao/fpt-cli/compare/v0.2.23...v0.2.24) (2026-04-01)


### Bug Fixes

* rename skill slug from fpt-cli-openclaw to fpt-cli ([08aca76](https://github.com/loonghao/fpt-cli/commit/08aca7661facd09193bdc7df9f5ce37c6e210961))

## [0.2.23](https://github.com/loonghao/fpt-cli/compare/v0.2.22...v0.2.23) (2026-04-01)


### Features

* add entity relationship delete endpoint ([#100](https://github.com/loonghao/fpt-cli/issues/100)) ([68ed3d8](https://github.com/loonghao/fpt-cli/commit/68ed3d8e8275407dc81e4aa9c3e71e71c308c99e))
* add entity relationship write (create/update) and entity share endpoints ([cd892b2](https://github.com/loonghao/fpt-cli/commit/cd892b2719feb6c19b1034eb8181965ab2263868))
* add entity.relationship, entity.update-last-accessed, followers.following, schema.entity-read, schema.field-revive endpoints ([0429ceb](https://github.com/loonghao/fpt-cli/commit/0429cebb7a524ec8e87467d6d536b485fe7d4b5f))
* add missing ShotGrid API endpoints, public API facade, tests & CI coverage ([bb4cde5](https://github.com/loonghao/fpt-cli/commit/bb4cde50879f333d5f8d586b7f8bce8387bf7bcf))
* add preferences update, note reply update/delete endpoints ([#98](https://github.com/loonghao/fpt-cli/issues/98)) ([189ba5c](https://github.com/loonghao/fpt-cli/commit/189ba5c9f12bd9f6df351dc556ca2fc2fadff7a6))
* add resumable bulk upsert with checkpoint/resume support ([#43](https://github.com/loonghao/fpt-cli/issues/43)) ([1773651](https://github.com/loonghao/fpt-cli/commit/17736517a3ac8c6f6f1a83cab0ab36becf573923))
* add schedule work-day-rules create/delete and thumbnail upload endpoints ([#112](https://github.com/loonghao/fpt-cli/issues/112)) ([ab6284b](https://github.com/loonghao/fpt-cli/commit/ab6284b160ba4571fe91ded9cab38b87eda4e4d2))
* add schema entity-update, entity-delete, and batch find-one commands ([88289bb](https://github.com/loonghao/fpt-cli/commit/88289bba014532fadc108c946f3fc40cda3b5395))
* add schema.field-read, work-schedule.update, entity.text-search, note.reply-create, entity.batch.revive APIs ([#71](https://github.com/loonghao/fpt-cli/issues/71)) ([85a30e9](https://github.com/loonghao/fpt-cli/commit/85a30e94719536e9275d5e31da6314de417b8272))
* add user.current, note.reply-read, and filmstrip.url CLI commands ([#94](https://github.com/loonghao/fpt-cli/issues/94)) ([11f35fb](https://github.com/loonghao/fpt-cli/commit/11f35fb000a41130265e7874ad84fd9a26022334))
* apply agent-dx best practices (inspect list, batch count, input hardening, SKILL.md) ([#115](https://github.com/loonghao/fpt-cli/issues/115)) ([e1240b9](https://github.com/loonghao/fpt-cli/commit/e1240b9565d06e07420a34b557fc961fc1789138))
* enrich all AppError callsites with structured details fields ([9065ff5](https://github.com/loonghao/fpt-cli/commit/9065ff5faae47f4d1d54cc2fd3e46d11a40dd8bd))
* expand shotgrid cli capabilities and release workflows ([f4d50d7](https://github.com/loonghao/fpt-cli/commit/f4d50d79d87d69e2a3504a6c067936b051862c3c))
* extend API coverage with entity count, batch summarize, and schema entity CRUD ([79e2f3d](https://github.com/loonghao/fpt-cli/commit/79e2f3d6b0dac7b44eac67fc15aed7a584a32442))
* extend REST API coverage with hierarchy expand, schedule, license, and preferences custom-entity ([#104](https://github.com/loonghao/fpt-cli/issues/104)) ([756d9ca](https://github.com/loonghao/fpt-cli/commit/756d9ca4016d7f23e0fb2676325a83715c8a6b1d))
* improve entity-link filter ergonomics in entity find ([78f645c](https://github.com/loonghao/fpt-cli/commit/78f645cad687bf010e44209e335115381f8c7f36)), closes [#42](https://github.com/loonghao/fpt-cli/issues/42)
* improve shotgrid cli docs and coverage ([3ed1f26](https://github.com/loonghao/fpt-cli/commit/3ed1f261afcd4d0fa8b7c03f5b4a6ff54c123f9c))


### Bug Fixes

* **#41-45:** exit codes, search normalization, entity-link DSL, batch upsert, retry/stats ([0f260d5](https://github.com/loonghao/fpt-cli/commit/0f260d50d55087b6ef5d1f7e910556a1bc53ff5e))
* add --locked to CI commands and update Cargo.lock ([024b4bc](https://github.com/loonghao/fpt-cli/commit/024b4bcef687f2e5c5fb477e2575db4400c8a926))
* **ci:** add build test job and update Cargo.lock ([9125559](https://github.com/loonghao/fpt-cli/commit/9125559426a6704a1af05b147fc52d1affadf652))
* **ci:** catch stale Cargo.lock before release ([3e96719](https://github.com/loonghao/fpt-cli/commit/3e967198655ee0d74677a67751d177adfe08558a))
* **ci:** enable cargo-workspace plugin for release-please ([4e850a9](https://github.com/loonghao/fpt-cli/commit/4e850a9c940a87bb623e935dff67be4a813951c0))
* **ci:** install cargo-hakari in quality workflow ([0914f8f](https://github.com/loonghao/fpt-cli/commit/0914f8f89f73c8edb24b95b326467a2934676e21))
* **ci:** publish release and skills from release-please ([9ae31aa](https://github.com/loonghao/fpt-cli/commit/9ae31aaf857621c2ecd90b83ddbe543bf35681ff))
* **ci:** publish release and skills from release-please ([c39377f](https://github.com/loonghao/fpt-cli/commit/c39377f078e3a6b9dfda10190f2b7d36d8ab3c25))
* **ci:** sync lockfile with workspace version ([a425815](https://github.com/loonghao/fpt-cli/commit/a4258156ba8bd7b981fe33e590c4f5e15a9808e4))
* correct release-please crate path ([0271971](https://github.com/loonghao/fpt-cli/commit/0271971dd58980734b0967729435790cb45b28e4))
* **deps:** update rust crate bit-vec to 0.9 ([c107d44](https://github.com/loonghao/fpt-cli/commit/c107d44babf6c3417d0112dd678ce4f2b9ddcfd1))
* **deps:** update rust crate lalrpop-util to 0.23 ([d861f4d](https://github.com/loonghao/fpt-cli/commit/d861f4d093f6a8fcfcc01182e8cb781836eae748))
* **deps:** update rust crate windows-sys-b21d60becc0929df to 0.61 ([79502ed](https://github.com/loonghao/fpt-cli/commit/79502ed9e277322834abcc744e06d469bb3f50a4))
* isolate auth_config_tests env and apply cargo fmt ([1d4de57](https://github.com/loonghao/fpt-cli/commit/1d4de57760f942b8711f74e5610159045f316792))
* normalize structured search payloads consistently across query commands ([16f02b8](https://github.com/loonghao/fpt-cli/commit/16f02b8bd79b838cfa776ecc5d87ce36430ea700)), closes [#44](https://github.com/loonghao/fpt-cli/issues/44)
* resolve config clear panic, capabilities version mismatch, and harden exit codes ([#48](https://github.com/loonghao/fpt-cli/issues/48), [#49](https://github.com/loonghao/fpt-cli/issues/49), [#45](https://github.com/loonghao/fpt-cli/issues/45)) ([26da0f7](https://github.com/loonghao/fpt-cli/commit/26da0f78486f6b4655d2544b04206f0749aa7e0a))
* **skills:** complete credential declarations in SKILL.md and install-and-auth.md ([6b4dbe5](https://github.com/loonghao/fpt-cli/commit/6b4dbe55c68bf8606bce038d200725e4bd7917a6))
* stabilize ci checks for pull request ([0e01a1c](https://github.com/loonghao/fpt-cli/commit/0e01a1cd4b9528a349839584eb85e24541517f6d))
* **test:** isolate infers_user_password_auth test from persisted config and env vars ([f04be7b](https://github.com/loonghao/fpt-cli/commit/f04be7b9478d3e6e4faf9126c86c7e7210acf848))
* unify release artifact publishing ([18b0476](https://github.com/loonghao/fpt-cli/commit/18b0476020ff0ba1662012bc0fc8ba773631e179))
* use candidates.contains() instead of iter().any() per clippy ([e543059](https://github.com/loonghao/fpt-cli/commit/e5430599dd75579b5e6b325212e86e811719a8de))
* use rustls-tls-native-roots to support enterprise MITM proxies ([#64](https://github.com/loonghao/fpt-cli/issues/64)) ([d3a0eda](https://github.com/loonghao/fpt-cli/commit/d3a0edaf37e1925848a5868f9d457d01506b8f3a))
* use vx uv for skill packaging workflows ([39e1e55](https://github.com/loonghao/fpt-cli/commit/39e1e556ea2eecc3321b579448110ae095160fc2))


### Code Refactoring

* cache env var reads with OnceLock and improve error context ([#107](https://github.com/loonghao/fpt-cli/issues/107)) ([86b8dc8](https://github.com/loonghao/fpt-cli/commit/86b8dc8f78ce50a9ccd8b8f498cca511cf54464f))
* consolidate self_update map_err helpers and add edge-case transport tests ([#92](https://github.com/loonghao/fpt-cli/issues/92)) ([e252079](https://github.com/loonghao/fpt-cli/commit/e2520796eec3f4626da3dfb1122833447e31076c))
* deduplicate filter normalization and relocate query param builder ([#88](https://github.com/loonghao/fpt-cli/issues/88)) ([6996dd5](https://github.com/loonghao/fpt-cli/commit/6996dd5ae71e57f44e31c43033119239f00ea96e))
* eliminate code duplication and improve type safety ([#84](https://github.com/loonghao/fpt-cli/issues/84)) ([a6ef06e](https://github.com/loonghao/fpt-cli/commit/a6ef06e9fd5c5c04ce0097bd7823b045b0a44f3b))
* extract batch result helpers, deduplicate retry logic, and tighten visibility ([#89](https://github.com/loonghao/fpt-cli/issues/89)) ([7a967fc](https://github.com/loonghao/fpt-cli/commit/7a967fcd0cf948d2a24b669c55434fd266bf1ec9))
* extract transport constants, add doc comments, simplify helpers, fix batch id display ([#108](https://github.com/loonghao/fpt-cli/issues/108)) ([abc7511](https://github.com/loonghao/fpt-cli/commit/abc75119ace3c5b6f1b1ba2848229d7f6867e15f))


### Documentation

* align skill install paths with installers ([7034e7d](https://github.com/loonghao/fpt-cli/commit/7034e7d8b941f1fb30722203d5974a04bcc245ce))
* harden OpenClaw skill install guidance ([d35bdd9](https://github.com/loonghao/fpt-cli/commit/d35bdd9077f50fa1f48fdc3fab27e0d6afce8bea))
* update READMEs with complete command reference for all 72 CLI commands ([#110](https://github.com/loonghao/fpt-cli/issues/110)) ([cf9f30f](https://github.com/loonghao/fpt-cli/commit/cf9f30fd11d3f3742154035eb3cd74903dee288e))


### Miscellaneous Chores

* code cleanup - narrow visibility, extract constant, fix httpmock 0.8 API, fix workspace-hack ([#103](https://github.com/loonghao/fpt-cli/issues/103)) ([f263ba6](https://github.com/loonghao/fpt-cli/commit/f263ba643660076840a5ee49e4fc5057701f7242))
* code cleanup — extract constants, deduplicate helpers, improve naming ([#76](https://github.com/loonghao/fpt-cli/issues/76)) ([a84a3d6](https://github.com/loonghao/fpt-cli/commit/a84a3d65aeacbe72fcac7c921717f4ecf42e7d82))
* code cleanup — remove dead code, fix backoff jitter, improve idioms ([#68](https://github.com/loonghao/fpt-cli/issues/68)) ([9d1e0ae](https://github.com/loonghao/fpt-cli/commit/9d1e0aeaf81a2b91d95d46d65e9987075bec2e36))
* code cleanup — remove unnecessary clones, use idiomatic patterns, fix panic paths ([#72](https://github.com/loonghao/fpt-cli/issues/72)) ([89e8a33](https://github.com/loonghao/fpt-cli/commit/89e8a330e2ff6e5aef00dd76065953a90c7ee25f))
* code cleanup round 4 — deduplicate errors and helpers\n\n- Extract bad_format closure in split_repository to eliminate duplicated error construction (self_update.rs)\n- Extract binary_not_found_error helper shared by extract_tar_gz_binary and extract_zip_binary (self_update.rs)\n- Add RestTransport::is_debug() to consolidate 3 redundant env var lookups (transport.rs)\n- Replace build_note_query_params with existing build_common_query_params from activity.rs (~30 lines removed) (note.rs)\n- Replace format allocation with in-place push for trailing newline (config.rs) ([#86](https://github.com/loonghao/fpt-cli/issues/86)) ([95b733f](https://github.com/loonghao/fpt-cli/commit/95b733f6c2742ff583478c215384f3b1ecc13800))
* deduplicate patterns and replace magic strings ([#109](https://github.com/loonghao/fpt-cli/issues/109)) ([ee015dc](https://github.com/loonghao/fpt-cli/commit/ee015dc11c1d331eb838142952df33f63991d524))
* deduplicate query helpers and remove unused code ([#70](https://github.com/loonghao/fpt-cli/issues/70)) ([5452752](https://github.com/loonghao/fpt-cli/commit/545275236fe45d9c2afb910b9d9285541a5d009c))
* dependency consistency and COMMANDS ordering cleanup ([#79](https://github.com/loonghao/fpt-cli/issues/79)) ([2d52ce6](https://github.com/loonghao/fpt-cli/commit/2d52ce693bc93579660956a6c94a6c2e79e78176))
* **deps:** bump rustls-webpki from 0.103.9 to 0.103.10 ([07880cb](https://github.com/loonghao/fpt-cli/commit/07880cb54b49fa0d89f7fdb09107ff75d269fc28))
* **deps:** bump tar from 0.4.44 to 0.4.45 ([e76208b](https://github.com/loonghao/fpt-cli/commit/e76208b020ccfe65484f9591d86d78149991f958))
* **deps:** update github artifact actions ([7592191](https://github.com/loonghao/fpt-cli/commit/7592191fc8955bfb2ae359b9ed2d53bcfc954337))
* **deps:** update github artifact actions ([d48fa69](https://github.com/loonghao/fpt-cli/commit/d48fa693ab4807625d4b19974df4e75109d24869))
* **deps:** update github artifact actions ([a8f937e](https://github.com/loonghao/fpt-cli/commit/a8f937ebd3dfa05f573f1262fb637f228e0fecd1))
* **deps:** update github artifact actions ([c96b0b6](https://github.com/loonghao/fpt-cli/commit/c96b0b6ed48393ada47d7c995218873aabe77f54))
* **deps:** update rust crate httpmock to 0.8.0 ([92cc582](https://github.com/loonghao/fpt-cli/commit/92cc5828d7a653a2a45bb99d101aeadd3bbea1b0))
* **deps:** update rust crate rstest to 0.26.0 ([199d2a7](https://github.com/loonghao/fpt-cli/commit/199d2a7c43b2d507e364acdc2961ca52290c5e46))
* **deps:** update rust crate zip to v8 ([2573e61](https://github.com/loonghao/fpt-cli/commit/2573e61757d2cae17c5b0099bc615d156344aee1))
* **deps:** update rust crate zip to v8.3.0 ([23d87a6](https://github.com/loonghao/fpt-cli/commit/23d87a623356926273e848822b8290b345a5ce7c))
* **deps:** update rust crate zip to v8.3.1 ([#62](https://github.com/loonghao/fpt-cli/issues/62)) ([f4294c1](https://github.com/loonghao/fpt-cli/commit/f4294c1b569706f068934f195f547075fd5836be))
* **deps:** update rust crate zip to v8.4.0 ([a5512b3](https://github.com/loonghao/fpt-cli/commit/a5512b3296b5517c7485c8d5f24bc4e1bc228ee0))
* enable chore type to bump version in release-please config ([dcf27b2](https://github.com/loonghao/fpt-cli/commit/dcf27b2b58f4d08c58aec2038366856733a948b8))
* extract magic strings into named constants and minor code hygiene ([#81](https://github.com/loonghao/fpt-cli/issues/81)) ([d2d7bea](https://github.com/loonghao/fpt-cli/commit/d2d7beacac4422816b18a1e57cbb5fbeffbe5ebb))
* extract preferences module, eliminate unnecessary clones, remove magic strings ([#111](https://github.com/loonghao/fpt-cli/issues/111)) ([6753254](https://github.com/loonghao/fpt-cli/commit/67532545d84e57b163b1cff8de0e4037c3076f2a))
* extract shared page-param parser and fix trailing whitespace ([#90](https://github.com/loonghao/fpt-cli/issues/90)) ([7dcc581](https://github.com/loonghao/fpt-cli/commit/7dcc581094e15601c0bda9dd73774178ca922878))
* fix release automation and add workspace hack ([ec5bc83](https://github.com/loonghao/fpt-cli/commit/ec5bc83fba65420adcfd791eaefbad9e1e4ef43f))
* **main:** release 0.2.0 ([6fc134b](https://github.com/loonghao/fpt-cli/commit/6fc134b6fad3e527f2eb79a3467ea8fc3050fba9))
* **main:** release v0.2.10 ([f98a597](https://github.com/loonghao/fpt-cli/commit/f98a59717b872edc0bf021ae7907c5e26741cd40))
* **main:** release v0.2.11 ([db63a87](https://github.com/loonghao/fpt-cli/commit/db63a87eb9f11ebfdcf558ad8bede81556cbb195))
* **main:** release v0.2.12 ([496945f](https://github.com/loonghao/fpt-cli/commit/496945fb860f9310a0b193c4f49dc18476a915df))
* **main:** release v0.2.13 ([02c6871](https://github.com/loonghao/fpt-cli/commit/02c68718287f11cf707ebe6e42fc8ae733664af4))
* **main:** release v0.2.13 ([94a0507](https://github.com/loonghao/fpt-cli/commit/94a05075635e7e0d59acd6803e9481ca9ce021b6))
* **main:** release v0.2.14 ([e6ab4ba](https://github.com/loonghao/fpt-cli/commit/e6ab4ba12654259f4306fda9a15ceadf1ed2f2c1))
* **main:** release v0.2.15 ([1ad3fc8](https://github.com/loonghao/fpt-cli/commit/1ad3fc877003441a6d237faeba08eae5871e3d19))
* **main:** release v0.2.16 ([275c8b3](https://github.com/loonghao/fpt-cli/commit/275c8b39f4c5d1d505892880e45ca11679192fc2))
* **main:** release v0.2.17 ([6c716dd](https://github.com/loonghao/fpt-cli/commit/6c716dda45a15a104542e5d60f1056cd0ceac211))
* **main:** release v0.2.18 ([e00d87e](https://github.com/loonghao/fpt-cli/commit/e00d87e52d172ccd4d5df659a856d66f2578a982))
* **main:** release v0.2.19 ([6b01575](https://github.com/loonghao/fpt-cli/commit/6b015759b750c34c9a84716d4ab4db8d33da123d))
* **main:** release v0.2.2 ([1f947a4](https://github.com/loonghao/fpt-cli/commit/1f947a4f2582107c03a906348fcf74a19c0f98a1))
* **main:** release v0.2.20 ([9e9075a](https://github.com/loonghao/fpt-cli/commit/9e9075a3f4155a8db0bff3b27e76fbb6ecb6b5b3))
* **main:** release v0.2.21 ([9342dbe](https://github.com/loonghao/fpt-cli/commit/9342dbecf7b51cda61cbe31fe60856a5a4c98942))
* **main:** release v0.2.22 ([09b485d](https://github.com/loonghao/fpt-cli/commit/09b485de243b44a98806a4d76345a429bbe96bf1))
* **main:** release v0.2.3 ([8a6bcbf](https://github.com/loonghao/fpt-cli/commit/8a6bcbf28b9fba3d8466e1a62e5b2eec98cc53dc))
* **main:** release v0.2.4 ([f379ce7](https://github.com/loonghao/fpt-cli/commit/f379ce71dc9efedf762bee06b856cd9b761a0522))
* **main:** release v0.2.5 ([0534b21](https://github.com/loonghao/fpt-cli/commit/0534b21f864cb2a862fe7da626f0ac77c356da53))
* **main:** release v0.2.6 ([889a388](https://github.com/loonghao/fpt-cli/commit/889a388e8127add280e41b66bee0c5f49e95cab0))
* **main:** release v0.2.7 ([2a80472](https://github.com/loonghao/fpt-cli/commit/2a8047246c4023f4628f29673ea10fedd6679eac))
* **main:** release v0.2.8 ([bde4917](https://github.com/loonghao/fpt-cli/commit/bde491706e86595ab61f60fa8f4d2b748b5f4915))
* **main:** release v0.2.9 ([215e176](https://github.com/loonghao/fpt-cli/commit/215e1763faf03ad034cde67e519c3f8a7ed55975))
* release v0.2.1 ([21f0c27](https://github.com/loonghao/fpt-cli/commit/21f0c27066ee2c20f8f7dd742c45e01d9fd73426))
* remove --locked flags from cargo commands ([a6b5419](https://github.com/loonghao/fpt-cli/commit/a6b5419b78086f560ff349a4761e0694b9314e6f))
* replace all Chinese strings/comments with English ([e938351](https://github.com/loonghao/fpt-cli/commit/e938351401d7ef635f125a4fb5685137d4954b5a))
* update dependencies (cargo update) ([12909a2](https://github.com/loonghao/fpt-cli/commit/12909a2072078a1f1d5bb5bcb711fef5ea3cfd57))

## [0.2.22](https://github.com/loonghao/fpt-cli/compare/v0.2.21...v0.2.22) (2026-03-26)


### Features

* add entity relationship delete endpoint ([#100](https://github.com/loonghao/fpt-cli/issues/100)) ([68ed3d8](https://github.com/loonghao/fpt-cli/commit/68ed3d8e8275407dc81e4aa9c3e71e71c308c99e))

## [0.2.21](https://github.com/loonghao/fpt-cli/compare/v0.2.20...v0.2.21) (2026-03-25)


### Features

* add entity relationship write (create/update) and entity share endpoints ([cd892b2](https://github.com/loonghao/fpt-cli/commit/cd892b2719feb6c19b1034eb8181965ab2263868))
* add preferences update, note reply update/delete endpoints ([#98](https://github.com/loonghao/fpt-cli/issues/98)) ([189ba5c](https://github.com/loonghao/fpt-cli/commit/189ba5c9f12bd9f6df351dc556ca2fc2fadff7a6))
* add user.current, note.reply-read, and filmstrip.url CLI commands ([#94](https://github.com/loonghao/fpt-cli/issues/94)) ([11f35fb](https://github.com/loonghao/fpt-cli/commit/11f35fb000a41130265e7874ad84fd9a26022334))

## [0.2.20](https://github.com/loonghao/fpt-cli/compare/v0.2.19...v0.2.20) (2026-03-25)


### Bug Fixes

* **deps:** update rust crate lalrpop-util to 0.23 ([d861f4d](https://github.com/loonghao/fpt-cli/commit/d861f4d093f6a8fcfcc01182e8cb781836eae748))

## [0.2.19](https://github.com/loonghao/fpt-cli/compare/v0.2.18...v0.2.19) (2026-03-24)


### Code Refactoring

* consolidate self_update map_err helpers and add edge-case transport tests ([#92](https://github.com/loonghao/fpt-cli/issues/92)) ([e252079](https://github.com/loonghao/fpt-cli/commit/e2520796eec3f4626da3dfb1122833447e31076c))


### Miscellaneous Chores

* extract shared page-param parser and fix trailing whitespace ([#90](https://github.com/loonghao/fpt-cli/issues/90)) ([7dcc581](https://github.com/loonghao/fpt-cli/commit/7dcc581094e15601c0bda9dd73774178ca922878))

## [0.2.18](https://github.com/loonghao/fpt-cli/compare/v0.2.17...v0.2.18) (2026-03-24)


### Code Refactoring

* deduplicate filter normalization and relocate query param builder ([#88](https://github.com/loonghao/fpt-cli/issues/88)) ([6996dd5](https://github.com/loonghao/fpt-cli/commit/6996dd5ae71e57f44e31c43033119239f00ea96e))
* eliminate code duplication and improve type safety ([#84](https://github.com/loonghao/fpt-cli/issues/84)) ([a6ef06e](https://github.com/loonghao/fpt-cli/commit/a6ef06e9fd5c5c04ce0097bd7823b045b0a44f3b))
* extract batch result helpers, deduplicate retry logic, and tighten visibility ([#89](https://github.com/loonghao/fpt-cli/issues/89)) ([7a967fc](https://github.com/loonghao/fpt-cli/commit/7a967fcd0cf948d2a24b669c55434fd266bf1ec9))


### Miscellaneous Chores

* code cleanup round 4 — deduplicate errors and helpers\n\n- Extract bad_format closure in split_repository to eliminate duplicated error construction (self_update.rs)\n- Extract binary_not_found_error helper shared by extract_tar_gz_binary and extract_zip_binary (self_update.rs)\n- Add RestTransport::is_debug() to consolidate 3 redundant env var lookups (transport.rs)\n- Replace build_note_query_params with existing build_common_query_params from activity.rs (~30 lines removed) (note.rs)\n- Replace format allocation with in-place push for trailing newline (config.rs) ([#86](https://github.com/loonghao/fpt-cli/issues/86)) ([95b733f](https://github.com/loonghao/fpt-cli/commit/95b733f6c2742ff583478c215384f3b1ecc13800))

## [0.2.17](https://github.com/loonghao/fpt-cli/compare/v0.2.16...v0.2.17) (2026-03-24)


### Miscellaneous Chores

* dependency consistency and COMMANDS ordering cleanup ([#79](https://github.com/loonghao/fpt-cli/issues/79)) ([2d52ce6](https://github.com/loonghao/fpt-cli/commit/2d52ce693bc93579660956a6c94a6c2e79e78176))
* extract magic strings into named constants and minor code hygiene ([#81](https://github.com/loonghao/fpt-cli/issues/81)) ([d2d7bea](https://github.com/loonghao/fpt-cli/commit/d2d7beacac4422816b18a1e57cbb5fbeffbe5ebb))

## [0.2.16](https://github.com/loonghao/fpt-cli/compare/v0.2.15...v0.2.16) (2026-03-24)


### Features

* add schema entity-update, entity-delete, and batch find-one commands ([88289bb](https://github.com/loonghao/fpt-cli/commit/88289bba014532fadc108c946f3fc40cda3b5395))

## [0.2.15](https://github.com/loonghao/fpt-cli/compare/v0.2.14...v0.2.15) (2026-03-23)


### Features

* add entity.relationship, entity.update-last-accessed, followers.following, schema.entity-read, schema.field-revive endpoints ([0429ceb](https://github.com/loonghao/fpt-cli/commit/0429cebb7a524ec8e87467d6d536b485fe7d4b5f))
* add schema.field-read, work-schedule.update, entity.text-search, note.reply-create, entity.batch.revive APIs ([#71](https://github.com/loonghao/fpt-cli/issues/71)) ([85a30e9](https://github.com/loonghao/fpt-cli/commit/85a30e94719536e9275d5e31da6314de417b8272))


### Miscellaneous Chores

* code cleanup — remove dead code, fix backoff jitter, improve idioms ([#68](https://github.com/loonghao/fpt-cli/issues/68)) ([9d1e0ae](https://github.com/loonghao/fpt-cli/commit/9d1e0aeaf81a2b91d95d46d65e9987075bec2e36))
* code cleanup — remove unnecessary clones, use idiomatic patterns, fix panic paths ([#72](https://github.com/loonghao/fpt-cli/issues/72)) ([89e8a33](https://github.com/loonghao/fpt-cli/commit/89e8a330e2ff6e5aef00dd76065953a90c7ee25f))
* deduplicate query helpers and remove unused code ([#70](https://github.com/loonghao/fpt-cli/issues/70)) ([5452752](https://github.com/loonghao/fpt-cli/commit/545275236fe45d9c2afb910b9d9285541a5d009c))

## [0.2.14](https://github.com/loonghao/fpt-cli/compare/v0.2.13...v0.2.14) (2026-03-23)


### Miscellaneous Chores

* **deps:** update rust crate zip to v8.4.0 ([a5512b3](https://github.com/loonghao/fpt-cli/commit/a5512b3296b5517c7485c8d5f24bc4e1bc228ee0))

## [0.2.13](https://github.com/loonghao/fpt-cli/compare/v0.2.12...v0.2.13) (2026-03-23)


### Bug Fixes

* use rustls-tls-native-roots to support enterprise MITM proxies ([#64](https://github.com/loonghao/fpt-cli/issues/64)) ([d3a0eda](https://github.com/loonghao/fpt-cli/commit/d3a0edaf37e1925848a5868f9d457d01506b8f3a))

## [0.2.12](https://github.com/loonghao/fpt-cli/compare/v0.2.11...v0.2.12) (2026-03-23)


### Miscellaneous Chores

* **deps:** update rust crate zip to v8.3.1 ([#62](https://github.com/loonghao/fpt-cli/issues/62)) ([f4294c1](https://github.com/loonghao/fpt-cli/commit/f4294c1b569706f068934f195f547075fd5836be))

## [0.2.11](https://github.com/loonghao/fpt-cli/compare/v0.2.10...v0.2.11) (2026-03-21)


### Miscellaneous Chores

* **deps:** bump rustls-webpki from 0.103.9 to 0.103.10 ([07880cb](https://github.com/loonghao/fpt-cli/commit/07880cb54b49fa0d89f7fdb09107ff75d269fc28))
* **deps:** bump tar from 0.4.44 to 0.4.45 ([e76208b](https://github.com/loonghao/fpt-cli/commit/e76208b020ccfe65484f9591d86d78149991f958))

## [0.2.10](https://github.com/loonghao/fpt-cli/compare/v0.2.9...v0.2.10) (2026-03-20)


### Miscellaneous Chores

* **deps:** update rust crate zip to v8.3.0 ([23d87a6](https://github.com/loonghao/fpt-cli/commit/23d87a623356926273e848822b8290b345a5ce7c))

## [0.2.9](https://github.com/loonghao/fpt-cli/compare/v0.2.8...v0.2.9) (2026-03-19)


### Features

* add resumable bulk upsert with checkpoint/resume support ([#43](https://github.com/loonghao/fpt-cli/issues/43)) ([1773651](https://github.com/loonghao/fpt-cli/commit/17736517a3ac8c6f6f1a83cab0ab36becf573923))
* improve entity-link filter ergonomics in entity find ([78f645c](https://github.com/loonghao/fpt-cli/commit/78f645cad687bf010e44209e335115381f8c7f36)), closes [#42](https://github.com/loonghao/fpt-cli/issues/42)


### Bug Fixes

* **#41-45:** exit codes, search normalization, entity-link DSL, batch upsert, retry/stats ([0f260d5](https://github.com/loonghao/fpt-cli/commit/0f260d50d55087b6ef5d1f7e910556a1bc53ff5e))
* isolate auth_config_tests env and apply cargo fmt ([1d4de57](https://github.com/loonghao/fpt-cli/commit/1d4de57760f942b8711f74e5610159045f316792))
* normalize structured search payloads consistently across query commands ([16f02b8](https://github.com/loonghao/fpt-cli/commit/16f02b8bd79b838cfa776ecc5d87ce36430ea700)), closes [#44](https://github.com/loonghao/fpt-cli/issues/44)
* resolve config clear panic, capabilities version mismatch, and harden exit codes ([#48](https://github.com/loonghao/fpt-cli/issues/48), [#49](https://github.com/loonghao/fpt-cli/issues/49), [#45](https://github.com/loonghao/fpt-cli/issues/45)) ([26da0f7](https://github.com/loonghao/fpt-cli/commit/26da0f78486f6b4655d2544b04206f0749aa7e0a))
* **skills:** complete credential declarations in SKILL.md and install-and-auth.md ([6b4dbe5](https://github.com/loonghao/fpt-cli/commit/6b4dbe55c68bf8606bce038d200725e4bd7917a6))


### Miscellaneous Chores

* replace all Chinese strings/comments with English ([e938351](https://github.com/loonghao/fpt-cli/commit/e938351401d7ef635f125a4fb5685137d4954b5a))

## [0.2.8](https://github.com/loonghao/fpt-cli/compare/v0.2.7...v0.2.8) (2026-03-17)


### Features

* enrich all AppError callsites with structured details fields ([9065ff5](https://github.com/loonghao/fpt-cli/commit/9065ff5faae47f4d1d54cc2fd3e46d11a40dd8bd))


### Bug Fixes

* **test:** isolate infers_user_password_auth test from persisted config and env vars ([f04be7b](https://github.com/loonghao/fpt-cli/commit/f04be7b9478d3e6e4faf9126c86c7e7210acf848))
* use candidates.contains() instead of iter().any() per clippy ([e543059](https://github.com/loonghao/fpt-cli/commit/e5430599dd75579b5e6b325212e86e811719a8de))

## [0.2.7](https://github.com/loonghao/fpt-cli/compare/v0.2.6...v0.2.7) (2026-03-17)


### Features

* add missing ShotGrid API endpoints, public API facade, tests & CI coverage ([bb4cde5](https://github.com/loonghao/fpt-cli/commit/bb4cde50879f333d5f8d586b7f8bce8387bf7bcf))


### Miscellaneous Chores

* enable chore type to bump version in release-please config ([dcf27b2](https://github.com/loonghao/fpt-cli/commit/dcf27b2b58f4d08c58aec2038366856733a948b8))
* remove --locked flags from cargo commands ([a6b5419](https://github.com/loonghao/fpt-cli/commit/a6b5419b78086f560ff349a4761e0694b9314e6f))
* update dependencies (cargo update) ([12909a2](https://github.com/loonghao/fpt-cli/commit/12909a2072078a1f1d5bb5bcb711fef5ea3cfd57))

## [0.2.6](https://github.com/loonghao/fpt-cli/compare/v0.2.5...v0.2.6) (2026-03-16)


### Bug Fixes

* **ci:** sync lockfile with workspace version ([a425815](https://github.com/loonghao/fpt-cli/commit/a4258156ba8bd7b981fe33e590c4f5e15a9808e4))

## [0.2.5](https://github.com/loonghao/fpt-cli/compare/v0.2.4...v0.2.5) (2026-03-16)


### Bug Fixes

* **ci:** catch stale Cargo.lock before release ([3e96719](https://github.com/loonghao/fpt-cli/commit/3e967198655ee0d74677a67751d177adfe08558a))

## [0.2.4](https://github.com/loonghao/fpt-cli/compare/v0.2.3...v0.2.4) (2026-03-16)


### Bug Fixes

* add --locked to CI commands and update Cargo.lock ([024b4bc](https://github.com/loonghao/fpt-cli/commit/024b4bcef687f2e5c5fb477e2575db4400c8a926))

## [0.2.3](https://github.com/loonghao/fpt-cli/compare/v0.2.2...v0.2.3) (2026-03-15)


### Bug Fixes

* **ci:** add build test job and update Cargo.lock ([9125559](https://github.com/loonghao/fpt-cli/commit/9125559426a6704a1af05b147fc52d1affadf652))

## [0.2.2](https://github.com/loonghao/fpt-cli/compare/v0.2.1...v0.2.2) (2026-03-15)


### Bug Fixes

* **ci:** enable cargo-workspace plugin for release-please ([4e850a9](https://github.com/loonghao/fpt-cli/commit/4e850a9c940a87bb623e935dff67be4a813951c0))

## [0.2.1](https://github.com/loonghao/fpt-cli/compare/v0.2.0...v0.2.1) (2026-03-15)


### Bug Fixes

* **ci:** publish release and skills from release-please ([9ae31aa](https://github.com/loonghao/fpt-cli/commit/9ae31aaf857621c2ecd90b83ddbe543bf35681ff))
* **ci:** publish release and skills from release-please ([c39377f](https://github.com/loonghao/fpt-cli/commit/c39377f078e3a6b9dfda10190f2b7d36d8ab3c25))
* unify release artifact publishing ([18b0476](https://github.com/loonghao/fpt-cli/commit/18b0476020ff0ba1662012bc0fc8ba773631e179))


### Documentation

* align skill install paths with installers ([7034e7d](https://github.com/loonghao/fpt-cli/commit/7034e7d8b941f1fb30722203d5974a04bcc245ce))
* harden OpenClaw skill install guidance ([d35bdd9](https://github.com/loonghao/fpt-cli/commit/d35bdd9077f50fa1f48fdc3fab27e0d6afce8bea))

## [0.2.0](https://github.com/loonghao/fpt-cli/compare/v0.1.0...v0.2.0) (2026-03-15)


### Features

* expand shotgrid cli capabilities and release workflows ([f4d50d7](https://github.com/loonghao/fpt-cli/commit/f4d50d79d87d69e2a3504a6c067936b051862c3c))
* improve shotgrid cli docs and coverage ([3ed1f26](https://github.com/loonghao/fpt-cli/commit/3ed1f261afcd4d0fa8b7c03f5b4a6ff54c123f9c))


### Bug Fixes

* stabilize ci checks for pull request ([0e01a1c](https://github.com/loonghao/fpt-cli/commit/0e01a1cd4b9528a349839584eb85e24541517f6d))
