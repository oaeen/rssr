# 项目执行约束（rssr）

## 工作流程
- 严格按 `PLAN.md` 的步骤推进。
- 每进入一个新步骤，先把当前阶段任务细化到 `TODO.md`。
- 每完成一项任务，立即在 `TODO.md` 标记为已完成。
- 任务执行过程中持续调整 `TODO.md`，保持与实际进度一致。

## 测试与验收
- 所有可测试的功能必须补充自动化测试（单元、集成或 E2E 至少一种）。
- 优先真实数据与真实流程，尽量避免无意义 mock。
- 每个阶段完成前必须通过该阶段对应测试。

## 提交策略
- 每完成一个小任务就执行一次 `git commit`。
- commit message 采用简洁动宾结构，例如：`feat: add opml importer`、`test: cover feed parser fixtures`。
- 严禁提交密钥与敏感配置；本地测试密钥仅放在被忽略文件中。

## 技术偏好
- 桌面端：Tauri 2 + Rust。
- 前端：React + TypeScript。
- 存储：SQLite。
- AI 接口：OpenAI Compatible（自定义 `base_url` / `api_key` / `model`）。
