# TODO（按 PLAN.md 执行）

## Step 1 项目初始化与工程骨架
- [x] 初始化 Git 仓库
- [x] 创建 `.gitignore`
- [x] 写入项目执行约束到 `AGENTS.md`
- [x] 生成 Tauri + React + TypeScript 脚手架
- [x] 细化 Rust 模块目录（feed/importer/subscription/llm/storage/sync）
- [x] 细化前端目录（app/pages/components/store/services）
- [x] 增加 Rust smoke tests（命令与模块初始化）
- [x] 增加前端 smoke tests（主页面渲染）
- [x] 运行 `pnpm install`
- [x] 运行测试并通过
- [x] 本地启动并验证基础页面可用

## Step 2 数据模型与持久化层
- [x] 设计 SQLite schema（sources/entries/llm_cache）与约束
- [x] 新增 migration 文件并接入启动迁移能力
- [x] 实现 `SourceRepository` 基础 CRUD（先做 upsert/list/delete）
- [x] 实现去重约束（feed_url 唯一）
- [x] 增加 migration 测试（表与约束存在）
- [x] 增加 repository 测试（幂等 upsert、list、delete）
- [x] Step 2 测试验收通过

## Step 3 Feed 抓取与解析引擎
- [x] 实现 RSS/Atom/RDF/JSON Feed 解析
- [x] ETag/Last-Modified 条件请求
- [x] 去重策略
- [x] fixtures 解析测试与重试测试

## Step 4 导入能力（OPML + 常见输入）
- [x] 下载并固化 OPML/XML 测试样本
- [x] 实现 OPML 导入
- [x] 实现 URL/JSON 导入
- [x] 导入预览与冲突处理
- [x] 导入流程测试（含幂等）

## Step 5 订阅链接管理
- [x] 初始化应用数据库（本地 sqlite 文件）
- [x] 暴露订阅命令：list/upsert/delete
- [x] 暴露导入命令：preview/import（复用 importer + repository）
- [x] 订阅管理前端页面联调（新增/删除/导入）
- [x] 批量启停同步（active 状态批量更新）
- [x] 管理界面交互测试与 E2E（至少导入+新增+删除）

## Step 6 阅读体验与性能优化
- [ ] Apple 风格 UI 基础主题与动画
- [ ] 阅读列表/详情布局
- [ ] 已读状态、过滤、搜索
- [ ] 性能与交互测试

## Step 7 自定义 LLM API Provider 层
- [x] 保存本地测试配置（`.env.local`：base_url/api_key/model）
- [ ] Provider 配置模型（base_url/api_key/model）
- [ ] OpenAI Compatible 请求实现
- [ ] 设置页连通性测试
- [ ] Provider 合约测试

## Step 8 翻译与总结流水线
- [ ] 实现 translate/summarize 命令
- [ ] 任务缓存与重复请求抑制
- [ ] 功能 E2E 测试

## Step 9 后台同步与容错
- [ ] 定时同步与指数退避
- [ ] 离线可读
- [ ] 同步异常恢复测试

## Step 10 发布工程与跨平台交付
- [ ] 配置构建脚本与 CI 基础
- [ ] 打包安装产物（当前平台）
- [ ] 安装包 smoke 验证
