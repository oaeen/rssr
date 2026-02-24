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
- [x] Apple 风格 UI 基础主题与动画
- [x] 阅读列表/详情布局
- [x] 已读状态、过滤、搜索
- [x] 性能与交互测试

## Step 7 自定义 LLM API Provider 层
- [x] 保存本地测试配置（`.env.local`：base_url/api_key/model）
- [x] Provider 配置模型（base_url/api_key/model）
- [x] OpenAI Compatible 请求实现
- [x] 设置页连通性测试
- [x] Provider 合约测试

## Step 8 翻译与总结流水线
- [x] 实现 translate/summarize 命令
- [x] 任务缓存与重复请求抑制
- [x] 功能 E2E 测试

## Step 9 后台同步与容错
- [x] 定时同步与指数退避
- [x] 离线可读
- [x] 同步异常恢复测试

## Step 10 发布工程与跨平台交付
- [x] 配置构建脚本与 CI 基础
- [x] 打包安装产物（当前平台）
- [x] 安装包 smoke 验证

## Iteration 2（同步优化 + Folo 风格 + 阅读页 AI）
- [x] 设计并实现异步可观测的同步机制（避免前端长时间阻塞）
- [x] 增加同步并发/批量/超时/重试配置，并放入设置页
- [x] 同步退避与状态查询能力补充测试
- [x] 重构主界面为更接近 Folo 的三栏布局风格
- [x] 将状态与配置集中到独立设置页
- [x] 在阅读页内提供 AI 总结/翻译卡片与结果展示
- [x] 前端回归测试与后端回归测试通过

## Iteration 3（设置独立窗口 + 渲染修复 + 列高度优化）
- [x] 移除主界面最左侧导航列
- [x] 新增“设置”按钮并打开独立设置窗口
- [x] 设置窗口模式路由（同一代码入口，按窗口模式渲染）
- [x] 修复文章详情 HTML 渲染（不再显示原始标签文本）
- [x] 优化订阅列和文章列高度，确保撑满到底部并独立滚动
- [x] 补充可测试逻辑的前端测试
- [x] 前端测试与构建通过

## Iteration 4（只做总结 + 后台双语标题 + 列表时间）
- [x] 分析翻译出现代码格式的原因，并调整策略为“抓网页正文后总结”
- [x] 后端总结流程优先抓取网页正文，失败时回退 feed 内容
- [x] 移除整文翻译命令与阅读页翻译入口，仅保留 AI 总结
- [x] 增加后台标题翻译能力，持久化 `translated_title`
- [x] 阅读页列表展示双语标题（中文标题 + 原标题）
- [x] 阅读页列表展示发布时间
- [x] 优化阅读三栏高度与滚动行为
- [x] 前后端测试通过
- [x] 构建与打包验证通过

## Iteration 5（设置内聚 + ACL 修复 + 三栏独立滚动）
- [x] 移除顶部 header，主界面改为内容区直接渲染
- [x] 取消新建设置窗口逻辑，修复 `create_webview_window` ACL 报错
- [x] 新增设置中心，并将“订阅与导入”并入设置内部
- [x] 阅读页侧栏增加设置入口（单窗口内切换）
- [x] 优化三栏滚动，确保订阅/文章/详情各自独立滚动
- [x] 前端测试通过
- [x] 构建验证通过

## Iteration 6（标题自动翻译稳定性）
- [x] 未翻译标题查询改为按时间升序处理（发布时间/创建时间）
- [x] 新增独立后台标题翻译轮询任务（不依赖同步触发）
- [x] 补充仓储顺序测试用例
- [ ] 回归测试与打包验证
