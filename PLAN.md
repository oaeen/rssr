# RSSR Desktop 实施计划（Step by Step）

## 0. 目标与约束
- 目标：实现一个类似 Folo 的跨平台 RSS 桌面订阅软件，支持 OPML 与常见 RSS/Atom/RDF/JSON Feed 导入、订阅管理、自定义 LLM API 的翻译和总结、优雅且高性能 UI。
- 平台：macOS / Windows / Linux（同一代码库）。
- 约束：所有可测试模块必须有对应自动化测试；先核心可用，再增强体验；严格遵循 KISS、YAGNI、DRY、SOLID。

## 1. 推荐技能（通过 `find-skills` 检索）
以下是与本任务最匹配、优先级较高的技能候选：

1. `nodnarbnitram/claude-code-extensions@tauri-v2`
2. `dchuk/claude-code-tauri-skills@testing-tauri-apps`
3. `pproenca/dot-skills@rust-testing`
4. `samhvw8/dot-claude@ui-design-system`
5. `hieutrtr/ai1-skills@e2e-testing`
6. `martinholovsky/claude-skills-generator@sqlite-database-expert`
7. `jackspace/claudeskillz@openai-api`

可选（RSS 方向参考）：

1. `brooksy4503/rss-agent-discovery@rss-agent-discovery`
2. `brooksy4503/rss-agent-viewer@rss-agent-viewer`

建议安装命令（按需）：

```bash
npx skills add nodnarbnitram/claude-code-extensions@tauri-v2 -g -y
npx skills add dchuk/claude-code-tauri-skills@testing-tauri-apps -g -y
npx skills add pproenca/dot-skills@rust-testing -g -y
npx skills add samhvw8/dot-claude@ui-design-system -g -y
npx skills add hieutrtr/ai1-skills@e2e-testing -g -y
npx skills add martinholovsky/claude-skills-generator@sqlite-database-expert -g -y
npx skills add jackspace/claudeskillz@openai-api -g -y
```

## 2. 技术栈（落地版）
- 桌面壳：`Tauri 2`
- 核心逻辑：`Rust`（`tokio` + `reqwest` + `serde`）
- 前端：`React + TypeScript + Vite + TanStack Query + Zustand`
- 数据库：`SQLite + sqlx + migration`
- Feed 解析：`feed-rs`（RSS/Atom/RDF）+ `opml`（OPML）+ `serde_json`（JSON Feed）
- LLM 接入：OpenAI-Compatible Provider（`base_url` / `api_key` / `model`）
- 凭据存储：`tauri-plugin-keyring`
- 日志与排障：`tracing`

## 3. 测试总策略（可测必测）
- Rust 单元测试：核心领域逻辑、解析、去重、队列、重试、缓存键。
- Rust 集成测试：数据库仓储、抓取流程、LLM Provider 合约、导入流程。
- 前端单元/组件测试：订阅管理、列表渲染、状态同步、错误态展示。
- E2E 测试：关键用户路径（导入、订阅管理、阅读、翻译、总结、设置 API）。
- 回归样例（Golden Fixtures）：多种 feed/opml 样本固定化，避免解析回归。
- CI 门禁：
  - 每个 PR 必须通过 `lint + test + build`。
  - 关键路径 E2E 必跑。
  - 覆盖率目标：Rust 核心模块 `>= 80%`，前端关键业务模块 `>= 70%`。

## 4. 分阶段实施（Step by Step）

## Step 1：项目初始化与工程骨架
- 交付物：
  - Tauri + React + TypeScript 基础工程。
  - Rust backend 模块结构：`feed`, `importer`, `subscription`, `llm`, `storage`, `sync`。
  - 前端页面结构：`Discover/Subscriptions/Reader/Settings`。
- 测试：
  - Rust smoke test（命令注册与模块初始化）。
  - 前端 smoke test（主页面渲染）。
- 完成门槛：
  - 本地可启动，基础页面可切换，测试通过。

## Step 2：数据模型与持久化层
- 交付物：
  - SQLite 表设计与迁移：`sources`, `feeds`, `entries`, `entry_content`, `llm_tasks`, `llm_cache`。
  - 仓储接口（Repository）与实现，屏蔽 SQL 细节。
- 测试：
  - migration up/down 测试。
  - 仓储 CRUD 测试（内存库/临时库）。
  - 约束测试（唯一键、外键、去重键）。
- 完成门槛：
  - 数据层 API 稳定，重复导入不会产生脏数据。

## Step 3：Feed 抓取与解析引擎
- 交付物：
  - 支持 RSS 2.0 / Atom / RDF / JSON Feed 的抓取与解析。
  - ETag/Last-Modified 条件请求，304 快速路径。
  - 标准化与去重（URL 规范化 + guid/链接 hash）。
- 测试：
  - fixtures 解析测试（每种格式至少 3 组样本：标准/缺字段/边界）。
  - 条件请求测试（200/304/超时/重试）。
  - 去重逻辑测试（同文多源、重复更新）。
- 完成门槛：
  - 解析覆盖常见源，失败可定位。

## Step 4：导入能力（OPML + 常见输入）
- 交付物：
  - OPML 导入（含嵌套 outline 分组）。
  - URL 列表导入（文本/JSON）。
  - 导入预览与冲突提示（已存在订阅不重复创建）。
- 测试：
  - OPML fixture 测试（嵌套、多语言、非法节点）。
  - 幂等性测试（重复导入结果一致）。
  - UI 导入流程 E2E（选择文件 -> 预览 -> 确认导入）。
- 完成门槛：
  - 大文件导入可用、可中断、可恢复提示。

## Step 5：订阅链接管理
- 交付物：
  - 增删改查、分组、启停同步、批量操作。
  - 订阅元数据展示（上次同步时间、失败次数）。
- 测试：
  - 前端组件测试（表单校验、批量动作）。
  - 仓储与命令接口集成测试。
  - E2E（新增订阅、编辑、删除、分组移动）。
- 完成门槛：
  - 管理操作稳定、状态一致、错误可恢复。

## Step 6：阅读体验与性能优化
- 交付物：
  - 双栏/三栏阅读布局。
  - 虚拟列表、懒加载、骨架屏。
  - 已读/未读、收藏、搜索与过滤。
- 测试：
  - 组件测试（筛选、排序、标记已读）。
  - E2E（长列表滚动、切换订阅源、状态保持）。
  - 性能基线测试（首屏与滚动帧率 smoke）。
- 完成门槛：
  - 万级条目下仍可流畅浏览。

## Step 7：自定义 LLM API Provider 层
- 交付物：
  - Provider 抽象：`base_url`, `api_key`, `model`, `timeout`, `retry`, `headers`。
  - 兼容 OpenAI Chat/Responses 风格请求。
  - 设置页 API 配置与连通性检测。
- 测试：
  - Provider 合约测试（mock server：成功/429/5xx/超时/无效 JSON）。
  - 配置校验测试（空字段、错误 URL、无权限 key）。
  - E2E（保存配置 -> 连通性测试 -> 生效）。
- 完成门槛：
  - 任意兼容 API 可接入，失败可解释。

## Step 8：翻译与总结流水线
- 交付物：
  - 对文章执行翻译与总结（手动触发 + 自动策略）。
  - 任务队列与结果缓存（避免重复调用）。
  - Prompt 模板管理（简洁可配置）。
- 测试：
  - 任务编排单元测试（重试、取消、幂等）。
  - 缓存命中测试（相同输入不重复调用）。
  - E2E（选择文章 -> 触发翻译/总结 -> 展示结果）。
- 完成门槛：
  - 功能稳定，成本可控，响应时间可接受。

## Step 9：后台同步与容错
- 交付物：
  - 定时刷新、指数退避、失败隔离。
  - 离线可读（本地缓存优先）。
- 测试：
  - 调度测试（fake clock）。
  - 网络异常恢复测试（断网/恢复/部分失败）。
  - 数据一致性测试（同步过程中 UI 读写）。
- 完成门槛：
  - 不因单源失败影响全局。

## Step 10：发布工程与跨平台交付
- 交付物：
  - macOS/Windows/Linux 构建脚本。
  - CI 工作流（lint/test/build/release artifacts）。
  - 崩溃日志与诊断信息收集策略。
- 测试：
  - CI matrix 构建测试（3 平台）。
  - 安装包 smoke test（启动、导入、阅读、LLM 调用）。
- 完成门槛：
  - 三平台可安装、可运行、核心功能一致。

## 5. 质量红线（DoD）
- 任一步骤未完成对应测试，不进入下一步骤。
- 新功能必须附带至少一类自动化测试（单元/集成/E2E 至少一种）。
- 解析、导入、LLM 调用相关缺陷必须追加回归用例。
- 性能劣化超过阈值（首屏、滚动、同步耗时）必须先修复再合入。

## 6. MVP 范围（首版必须有）
- 跨平台打包运行（Tauri）。
- OPML + RSS/Atom/RDF/JSON Feed 导入。
- 订阅管理（增删改查 + 分组 + 手动刷新）。
- 文章阅读（列表 + 详情 + 已读状态）。
- 自定义 API 的翻译与总结（至少 1 个可配置 Provider）。
- 覆盖关键路径自动化测试与 CI 门禁。

## 7. 非目标（当前阶段不做，避免过度设计）
- 云端账号体系与多端实时同步。
- 推荐算法与社交功能。
- 复杂插件系统。
- 全量 AI 自动分类/标签体系。
