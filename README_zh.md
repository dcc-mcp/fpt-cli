## `fpt-cli`

[English](README.md) | [简体中文](README_zh.md)

一个面向 **OpenClaw、AI agent 与自动化工作流** 的 Rust 版 Autodesk Flow Production Tracking（**ShotGrid / FPT**）CLI。

`fpt-cli` 明确采用 **CLI-first** 设计。它的目标不仅是提供可用的 ShotGrid/FPT 操作命令，还要为 agent 提供稳定的命令契约，让编排更直接，并通过结构化 JSON 输出把可重复交互下沉到显式 CLI 命令里，从而减少不必要的 **MCP token 消耗**。

这个仓库同时也是对文章 [MCP Is Dead, Long Live the CLI](https://ejholmes.github.io/2026/02/28/mcp-is-dead-long-live-the-cli.html) 中相关观点的一次实践验证与持续探索。

它也是我围绕 MCP 能力做验证与对比尝试的主要项目；如果你想看一个更直接的 MCP 方案，可以参考 [shotgrid-mcp-server](https://github.com/loonghao/shotgrid-mcp-server)。


### 当前状态

当前仓库已实现对 ShotGrid REST 与 RPC API 的近完整覆盖：

- **Rust workspace** 拆分为专注的 crate（`fpt-cli`、`fpt-core`、`fpt-domain`）
- **CLI-first** 命令树，**76 个已注册命令**（全部实现）
- 面向自动化的 **结构化 JSON 输出**，已作为默认输出格式
- 命令级 **capability / inspect** 发现接口，包含 `inspect list` 用于 agent 模式内省
- **Agent DX 最佳实践**：实体类型输入加固（拒绝注入 `?`/`#` 及控制字符），防止 hallucination 构造恶意 URL
- **REST transport** 覆盖认证、schema、entity CRUD、关系、关注者、层级导航、活动流、事件日志、偏好设置、排程、许可证、上传/下载、缩略图
- **完整的 schema 管理**：entity type 与 field 的创建、读取、更新、删除与恢复
- 面向高频"只取第一条"场景的 **`entity.find-one`**
- 用于便捷记录计数的 **`entity.count`** 封装
- 用于跨多实体类型一次性计数的 **`entity.batch.count`**
- 用于服务端聚合统计与分组汇总的 **`entity.summarize`**
- 支持 `additional_filter_presets` 的 **`entity.find` 结构化 `_search`**
- 通过 **`entity batch`** 实现的 client-side 批量 CRUD 编排，包含 **`batch.upsert`** 和 **`batch.summarize`**
- **受控 batch 并发** 与稳定有序结果返回
- 写操作的 **dry-run** 请求计划输出
- **三种认证模式**：script、user password、session token
- **进程内 access token 复用**，降低重复认证开销
- **服务器信息**、**许可证**、**活动流**、**事件日志**、**层级导航** 与 **排程** 端点
- **关注者管理**：列出、关注、取消关注、查看用户关注列表
- **Note 线程管理**：线程列表、回复 CRUD
- **上传/下载 URL 生成** 与 **缩略图/filmstrip URL** 获取
- **持久化 CLI 配置**：`config get/set/clear/path`
- 位于 `skills/fpt-cli` 的 **可发布 OpenClaw skill 包**，遵循 Agent DX 最佳实践

### 开发环境

项目中的所有环境都应通过 **`vx`** 管理，命令集合通过仓库里的 **`justfile`** 暴露。


```bash
vx setup
vx just test
vx just capabilities
```

### 安装

预构建发布产物当前覆盖：

- **Linux**：`x86_64-unknown-linux-gnu`
- **Windows**：`x86_64-pc-windows-msvc`
- **macOS**：`x86_64-apple-darwin`、`aarch64-apple-darwin`

可通过 HTTPS 直接安装最新版本：

```bash
curl -fsSL https://raw.githubusercontent.com/dcc-mcp/fpt-cli/main/scripts/install.sh | sh

```

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/dcc-mcp/fpt-cli/main/scripts/install.ps1 | iex"

```

可选安装环境变量：

- `FPT_INSTALL_VERSION`：安装指定版本而不是 `latest`
- `FPT_INSTALL_DIR`：覆盖安装目录
- `FPT_INSTALL_REPOSITORY`：覆盖 GitHub 仓库（格式：`owner/repo`）

### 自更新

已安装的二进制可以原地自更新：

```bash
fpt self-update --check --output pretty-json
fpt self-update
fpt self-update --version 0.1.0
```

说明：

- `self-update` 会通过 HTTPS 拉取 GitHub Releases
- 它会自动选择与当前主机平台匹配的发布产物
- 如果 release 中包含 `fpt-checksums.txt`，替换前会先校验压缩包摘要

### OpenClaw skill

仓库内包含一个可发布的 OpenClaw skill：`skills/fpt-cli`。

发布后可通过 ClawHub 安装：

```bash
npx clawhub@latest install fpt-cli
```

在仓库本地打包整个 `skills/` 目录下的所有 skill：

```bash
vx just package-skills
```

如果只想打包 OpenClaw skill，也保留了单 skill 别名：

```bash
vx just package-openclaw-skill
```

`generate-skills.yml` 会负责为 `skills/` 下的所有 skill 生成 zip 产物，`clawhub.yml` 会在 PR 上执行 dry-run sync，并在 `main` push 时同步整个 `skills/` 根目录。

### 发布自动化

版本管理通过 **`release-please`** 完成：

- 向 `main` 推送 conventional commits
- `release-please` 创建或更新 release PR
- 合并 release PR 后生成版本 tag 和 GitHub release 元数据
- `release-please.yml` 会直接调用可复用的 `release.yml` workflow，为新生成的 tag 发布跨平台 CLI 二进制
- `generate-skills.yml` 会为 `skills/` 下的所有 skill 生成产物
- `clawhub.yml` 会在 PR 上执行 dry-run sync，在 `main` push 时同步 `skills/` 根目录，并且也会被 `release-please.yml` 直接调用，因此 release 时发布到 ClawHub 不再依赖第二次 tag 触发

`RELEASE_PLEASE_TOKEN` 是可选的。若已配置，`release-please` 会优先使用它；但下游的 release 构建与 ClawHub 发布已经不再依赖 tag 触发的 workflow 级联。

要启用 ClawHub 发布，请配置 `CLAWHUB_TOKEN`。


### Pre-commit

仓库根目录包含 `.pre-commit-config.yaml`，并通过 `vx` 封装 hook 管理：

```bash
vx just pre-commit-install
vx just pre-commit-run
```

当前配置的 hooks：

- `pre-commit`：whitespace / merge-conflict 检查、`vx just fmt-check`、`vx just lint`
- `pre-push`：`vx just test`

### 认证环境变量



CLI 默认优先使用 `FPT_*` 前缀；当 `FPT_*` 缺失时，也支持回退读取 `SG_*`。

- `FPT_SITE` / `SG_SITE`
- `FPT_AUTH_MODE` / `SG_AUTH_MODE`：`script` / `user_password` / `session_token`
- `FPT_SCRIPT_NAME` / `SG_SCRIPT_NAME`
- `FPT_SCRIPT_KEY` / `SG_SCRIPT_KEY`
- `FPT_USERNAME` / `SG_USERNAME`
- `FPT_PASSWORD` / `SG_PASSWORD`
- `FPT_AUTH_TOKEN` / `SG_AUTH_TOKEN`：站点启用 2FA 时可选
- `FPT_SESSION_TOKEN` / `SG_SESSION_TOKEN`
- `FPT_API_VERSION` / `SG_API_VERSION`（可选，默认 `v1.1`）

### 认证模式

- **script**：使用 `script_name + script_key` 走 `client_credentials`
- **user_password**：使用 `username + password` 走 `password` grant
- **session_token**：使用已有 `session_token` 走 `session_token` grant

如果没有显式传入 `--auth-mode`，CLI 会基于已有输入自动推断：

- **优先 user password**：只要出现 `username`、`password` 或 `auth_token` 之一
- **其次 session token**：当传入 `session_token` 时使用
- **否则 script**：回退到 `script_name + script_key`

### 已实现命令

```bash
# 核心
fpt capabilities --output json
fpt inspect command entity.update --output json
fpt inspect list --output json

# 认证
fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode script --script-name bot --script-key xxx
fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode user-password --username user@example.com --password secret
fpt auth test --site https://example.shotgrid.autodesk.com --auth-mode session-token --session-token xxx

# 服务器
fpt server info --site ...

# Schema — entity type
fpt schema entities --site ...
fpt schema entity-read Shot --site ...
fpt schema entity-create --input @entity_def.json --site ...
fpt schema entity-update CustomEntity01 --input @entity_props.json --site ...
fpt schema entity-delete CustomEntity01 --site ...
fpt schema entity-revive CustomEntity01 --site ...

# Schema — 字段
fpt schema fields Shot --site ...
fpt schema field-read Shot code --site ...
fpt schema field-create Shot --input '{"data_type":"text","properties":[{"property_name":"name","value":"Custom Field"}]}' --site ...
fpt schema field-update Shot sg_custom_field --input @field_props.json --site ...
fpt schema field-delete Shot sg_custom_field --site ...
fpt schema field-revive Shot sg_custom_field --site ...

# Entity CRUD
fpt entity get Shot 123 --site ...
fpt entity find Asset --input @query.json --site ...
fpt entity find Asset --filter-dsl "sg_status_list == 'ip' and (code ~ 'bunny' or id > 100)" --site ...
fpt entity find-one Shot --input @query.json --site ...
fpt entity summarize Version --input @summaries.json --site ...
fpt entity count Shot --filter-dsl "sg_status_list == 'ip'" --output json
fpt entity create Version --input @payload.json --dry-run
fpt entity update Task 42 --input @patch.json --dry-run
fpt entity delete Playlist 99 --dry-run
fpt entity revive Shot 860 --dry-run
fpt entity text-search --input '{"text":"hero shot"}' --site ...
fpt entity update-last-accessed 123 --site ...

# Entity 关系
fpt entity relationship Shot 123 --field assets --site ...
fpt entity relationship-create Shot 123 --field assets --input '{"data":[{"type":"Asset","id":7}]}' --site ...
fpt entity relationship-update Shot 123 --field assets --input '{"data":[{"type":"Asset","id":10}]}' --site ...
fpt entity relationship-delete Shot 123 --field assets --input '{"data":[{"type":"Asset","id":7}]}' --site ...
fpt entity share Shot 123 --input '{"entities":[{"type":"Project","id":85}]}' --site ...

# Entity 批量操作
fpt entity batch get Shot --input '{"ids":[101,102],"fields":["code","sg_status_list"]}' --output json
fpt entity batch find Asset --input @batch_queries.json --output json
fpt entity batch find-one Shot --input @batch_queries.json --output json
fpt entity batch create Version --input @batch_payloads.json --dry-run --output json
fpt entity batch update Task --input @batch_updates.json --dry-run --output json
fpt entity batch delete Playlist --input '{"ids":[99,100]}' --dry-run --output json
fpt entity batch revive Shot --input '{"ids":[860,861]}' --dry-run --output json
fpt entity batch upsert Shot --input @items.json --key code --on-conflict skip --dry-run --output json
fpt entity batch summarize --input @batch_summaries.json --output json
fpt entity batch count --input '["Shot","Asset","Task"]' --output json

# 关注者
fpt followers list Shot 123 --site ...
fpt followers follow Shot 123 --input '{"type":"HumanUser","id":456}' --site ...
fpt followers unfollow Shot 123 --input '{"type":"HumanUser","id":456}' --site ...
fpt followers following 456 --site ...

# Notes
fpt note threads 456 --site ...
fpt note reply-create 456 --input '{"content":"Looks great!"}' --site ...
fpt note reply-read 456 789 --site ...
fpt note reply-update 456 789 --input '{"content":"Updated reply"}' --site ...
fpt note reply-delete 456 789 --site ...

# 用户
fpt user current --site ...
fpt user current --user-type api --site ...

# 层级导航
fpt hierarchy search --input @hierarchy_query.json --site ...
fpt hierarchy expand --input @expand_query.json --site ...

# 排程
fpt work-schedule read --input @schedule.json --site ...
fpt work-schedule update --input '{"date":"2026-04-01","working":false}' --site ...
fpt schedule work-day-rules --site ...
fpt schedule work-day-rules-read 42 --site ...
fpt schedule work-day-rules-update 42 --input '{"date":"2026-04-01","is_working":false}' --site ...

# 活动与事件日志
fpt activity stream Shot 123 --site ...
fpt event-log entries --site ...

# 偏好设置
fpt preferences get --site ...
fpt preferences update --input '{"name":"value"}' --site ...
fpt preferences custom-entity --input '{"entity_type":"CustomEntity01"}' --site ...

# 许可证
fpt license get --site ...

# 上传 / 下载 / 缩略图
fpt upload url Shot 123 sg_uploaded_movie movie.mp4 --site ...
fpt download url Shot 123 sg_uploaded_movie --site ...
fpt thumbnail url Asset 55 --site ...
fpt filmstrip url Version 456 --site ...

# 自更新
fpt self update --check --output pretty-json
fpt self update
fpt self update --version 0.1.0

# 配置
fpt config get --output pretty-json
fpt config path
fpt config set --site https://example.shotgrid.autodesk.com --auth-mode script --script-name bot --script-key xxx
fpt config clear --all
fpt config clear --fields site,auth-mode
```

### 批量 CRUD

`entity batch` 提供批量 get / find / find-one / create / update / delete / revive / upsert / summarize / count 工作流。
当前实现是**在 CLI 侧编排已有 REST CRUD 端点**，统一返回 `results` 数组，其中每一项都会携带自己的 `ok` 状态以及 `response` 或 `error`。

输入约定：

- **`entity batch get`**：`[1,2,3]` 或 `{"ids":[1,2,3],"fields":["code"]}`
- **`entity batch find`**：`[{...query1...},{...query2...}]` 或 `{"requests":[...]}`
- **`entity batch find-one`**：与 `batch find` 相同，但每个查询只返回第一条匹配记录
- **`entity batch create`**：`[{...body1...},{...body2...}]` 或 `{"items":[...]}`
- **`entity batch update`**：`[{"id":42,"body":{...}}, {"id":43,"body":{...}}]` 或 `{"items":[...]}`
- **`entity batch delete`**：`[42,43]` 或 `{"ids":[42,43]}`
- **`entity batch revive`**：`[860,861]` 或 `{"ids":[860,861]}`
- **`entity batch upsert`**：`[{...body...}]` 或 `{"items":[...]}` 配合 `--key` 字段与 `--on-conflict` 策略
- **`entity batch summarize`**：`[{"entity":"Shot","payload":{...}}]` 或 `{"requests":[...]}`
- **`entity batch count`**：`["Shot","Asset","Task"]` 或 `{"entities":["Shot","Asset"]}` 或 `[{"entity":"Task","filters":[...]}]`

说明：

- **批量 create / update / delete / revive 支持 `--dry-run`**
- **批量 delete 真实执行仍要求显式传入 `--yes`**
- 同一次 CLI 进程中的 batch 子请求会**复用 access token**
- 同一次 CLI 进程中的 batch 子请求会以**受控并发**方式执行，默认并发度为 `8`
- 可通过 **`FPT_BATCH_CONCURRENCY`** 调整并发度；传入 `1` 可退回串行执行
- **`entity batch count`** 可在一次调用中统计多个实体类型的记录数，避免多次调用 `entity.count`，降低 round-trip 次数和上下文窗口开销

### 复杂过滤 DSL 与结构化 search

`entity find` 支持通过 `--filter-dsl`（或在 `--input` JSON 里传入 `filter_dsl`）描述复杂过滤条件。
它也支持原生 `search` 对象以及顶层 `additional_filter_presets`，用于直接构造 ShotGrid REST `_search` 请求体。
当使用 DSL 时，CLI 会自动切换到 ShotGrid REST 的 `_search` 端点。


DSL 支持：

- 字段路径：`field` / `linked.field`
- 逻辑运算：`and` / `or` / `(...)`
- 比较运算符：`==`、`!=`、`>`、`>=`、`<`、`<=`、`~`（映射为 `contains`）
- 关键字运算符：例如 `in`、`not in`、`starts_with`（按原样透传给 ShotGrid）
- 值类型：字符串、数字、布尔、`null`、数组

示例：

```bash
fpt entity find Asset --filter-dsl "sg_status_list == 'ip' and (code ~ 'bunny' or id > 100)"
```

> `filters` 与 `filter_dsl` 不能同时使用。

### 测试覆盖

当前测试覆盖分为三层，3 个 crate 中共 **97+ 个测试**：

- **App 编排测试**：`auth.test`、`server.info`、`schema.entities/fields/field-read/field-create/field-update/field-delete/field-revive/entity-read/entity-create/entity-update/entity-delete/entity-revive`、`entity.get/find/find-one/count/create/update/delete/revive/text-search/update-last-accessed`、`entity.batch.*`（get、find、find-one、create、update、delete、revive、upsert、summarize）、`entity.relationship/relationship-create/relationship-update/relationship-delete`、`entity.share`、`work-schedule.read/update`、`schedule.work-day-rules/work-day-rules-update`、`note.threads/reply-create/reply-read/reply-update/reply-delete`、`preferences.get/update/custom-entity`、`followers.list/follow/unfollow/following`、`user.current`、`hierarchy.search/expand`、`activity.stream`、`event-log.entries`、`license.get`、`upload.url`、`download.url`、`thumbnail.url`、`filmstrip.url`
- **REST transport 测试**：OAuth token 获取、schema/entity 路由映射、`_search` 切换、写操作 method 映射、错误分类、token 复用、所有 schema field/entity CRUD 端点、所有 follower 端点、所有 note 端点、所有 relationship 端点、hierarchy/activity/event-log/preferences/schedule/license/upload/download/thumbnail/filmstrip 端点、current user 端点
- **CLI 合约测试**：capabilities 输出验证、inspect 命令验证、dry-run plan 输出、policy-blocked delete、config CRUD、hierarchy/license/preferences 合约存在性

开发时建议优先执行：

```bash
vx just test
```

### OpenClaw 站点联调示例

建议优先使用环境变量，而不是把 secret 直接写进 shell 历史。

```powershell
$env:FPT_SITE = "https://openclaw.shotgrid.autodesk.com"
$env:FPT_AUTH_MODE = "user_password"
$env:FPT_USERNAME = "user@example.com"
$env:FPT_PASSWORD = "your-password"
vx cargo run -p fpt-cli -- auth test --output pretty-json
```

调试 `scripts/local_count_projects.ps1` 时，也可以在仓库根目录放一个 `.env` 文件。脚本会自动读取，并且不会覆盖当前 shell 已有的环境变量。

```dotenv
FPT_SITE=https://openclaw.shotgrid.autodesk.com
FPT_AUTH_MODE=script
FPT_SCRIPT_NAME=openclaw
FPT_SCRIPT_KEY="your-script-key"
```

```powershell
pwsh -File .\scripts\local_count_projects.ps1 -AuthMode script -VerbosePage
```

> 建议通过环境变量或 `.env` 传递 secret，不要直接放到命令行参数里。
> 在 Windows 下，`^`、`&`、`!`、`%` 等字符可能在 shell / 进程启动链路中被转义或吞掉。
> 当前脚本只传 `--auth-mode`，凭据由 CLI 从环境变量读取。
> 其直连预检使用 `Invoke-WebRequest -SkipHttpErrorCheck`，因此即使 ShotGrid 返回 `400`，也会尽量输出响应正文，方便诊断。

如果站点启用了双因素认证，还可以额外设置：

```powershell
$env:FPT_AUTH_TOKEN = "123456"
vx cargo run -p fpt-cli -- auth test --output pretty-json
```

### Agent DX 最佳实践

`fpt-cli` 遵循 [You Need to Rewrite Your CLI for AI Agents](https://justin.poehnelt.com/posts/rewrite-your-cli-for-ai-agents/) 中描述的 Agent DX 原则：

**1. 运行时模式内省——不要猜测**

```bash
# 获取所有命令名（低开销，无需全量 capabilities 载荷）
fpt inspect list --output json

# 只获取你真正需要的命令的契约
fpt inspect command entity.batch.count --output json
```

**2. 针对 hallucination 的输入加固**

CLI 会对实体类型名进行校验，拒绝以下情况：
- 包含 `?` 或 `#` 的名称（查询参数 / 片段标识符注入）
- 包含控制字符（低于 ASCII 0x20）的名称

这可以防止 hallucinate 的输入（如 `entity get "Shot?fields=code" 123`）静默构造出格式错误的 URL。

**3. 上下文窗口纪律**

用 `inspect list` 获取命令名，再用 `inspect command <name>` 只获取你需要的契约。
用 `entity batch count` 一次统计多个实体类型，而不是循环调用。
通过 `fields` 参数只请求你需要的字段。

**4. 写操作安全**

每个变更命令都支持 `--dry-run`，在执行前预览请求计划。
破坏性操作（delete）需要显式传入 `--yes`。

**5. OpenClaw skill**

`skills/fpt-cli/SKILL.md` 编码了所有无法从 `--help` 推断出的 Agent DX 不变量：
- 模式内省模式
- 输入加固规则
- 上下文窗口纪律
- 写操作安全协议

### 设计原则

- **CLI 契约独立于底层 transport 实现**
- **默认 JSON 输出，便于 agent 集成**
- **`--output toon`** 与 **`--output pretty-json`** 仍然保留，可在需要特定展示格式时显式指定

- **写操作支持 `--dry-run`**
- **未来即使新增 REST 之外的 transport，也不应破坏 OpenClaw 面向的 CLI 契约**
