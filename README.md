# RSSR

基于 Tauri + React + TypeScript 的跨平台 RSS 阅读器。

## 开发环境

- Node.js 22
- pnpm 10
- Rust stable
- Tauri 2

## 本地开发

```bash
pnpm install
pnpm tauri dev
```

## CI / CD 与 Release 策略

- `push` 到 `main` 会触发 CI：
  - 三平台测试：`ubuntu-latest` / `windows-latest` / `macos-latest`
  - 三平台安装包构建：`deb` / `AppImage` / `exe` / `msi` / `dmg`
- CI 全部通过后，自动发布 GitHub Release。

### 版本命名规则（语义化版本）

- Release Tag 格式：`vX.Y.Z`
- 版本号来源：`src-tauri/tauri.conf.json` 的 `version` 字段
- Release 名称格式：`RSSR vX.Y.Z`

发布新版本时，先更新 `src-tauri/tauri.conf.json` 中的 `version`，再推送到 `main`。

### Changelog

- Release 使用 GitHub 自动生成变更日志（`generate_release_notes: true`）。
- 每次发布会自动附上本次版本的变更摘要与贡献记录。
