# Repository Guidelines（仓库指南）

## 项目结构与模块组织

EasyTier 是一个 Rust 工作区，默认成员为 `easytier`（核心程序与 CLI）和 `easytier-web`（Web 服务）。通用网络与协议代码位于 `easytier-core/` 和 `easytier-proto/`。Tauri 桌面端与移动端位于 `easytier-gui/`；Web 包位于 `easytier-web/frontend*`；FFI、精简客户端和运行状态服务等辅助项目位于 `easytier-contrib/`。Protobuf 源文件放在 `easytier-proto/proto/`，设计文档放在 `docs/`，构建与安装脚本放在 `script/`，文档图片放在 `assets/`。

## 构建、测试与开发命令

Rust 版本由 `rust-toolchain.toml` 固定为 1.95；开发环境还需要 Node.js 21+、pnpm 9+、LLVM/Clang 和 `protoc`。

```bash
pnpm install                                         # 安装工作区依赖
cargo build --release                                # 构建默认 Rust 成员
cargo run -p easytier --bin easytier-core -- --help  # 本地运行核心 CLI
pnpm --dir easytier-gui dev                          # 启动 Tauri/Vite 前端
pnpm -r build                                        # 构建所有前端包
```

## 编码风格与命名约定

Rust 代码使用 `cargo fmt --all` 格式化。模块和函数使用 `snake_case`，类型使用 `UpperCamelCase`，常量使用 `SCREAMING_SNAKE_CASE`。Vue/TypeScript 遵循现有项目风格：组件文件使用 PascalCase，工具模块使用小写命名，并遵守 ESLint 格式规则。修改 GUI 后运行 `pnpm --dir easytier-gui lint`。应修改 `.proto` 源文件或代码生成器，不要直接编辑生成文件。

## 测试指南

提交前运行以下检查：`cargo fmt --all -- --check`、`cargo clippy --all-targets --features full --all -- -D warnings`，以及 `cargo test --no-default-features --features full --verbose`。前端共享库使用 Vitest，运行 `pnpm --dir easytier-web/frontend-lib test`。网络集成测试可能需要 Linux bridge、iptables、UPnP 和 IPv6 环境；测试应保持隔离且可重复执行。

## easytier-web 重构目标

针对 `easytier-web` 的重构应保持后端 API 和现有配置兼容，并围绕以下目标推进：

1. **支持 OAuth 2.0/OIDC 登录。** 应兼容 Authelia 等支持 OAuth 2.0/OIDC 的身份提供商，优先采用标准授权码流程和 PKCE。当前项目已有 `src/restful/oidc.rs`、状态校验和会话登录基础；扩展时应完善 Issuer、Client ID/Secret、Redirect URI、Scope 与用户字段映射配置，以及登录、回调、退出、账号关联、错误处理和安全 Cookie，并补充认证失败与重放攻击测试。
2. **采用现代 Web UI 技术改善展示。** 在 Vue 3、Vite、TypeScript 和 Tailwind CSS 基础上建立统一的设计系统，优化响应式布局、暗色模式、可访问性、加载/空状态/错误状态、表单反馈和网络拓扑数据可视化。公共组件优先放入 `frontend-lib/src/components/`，避免 `frontend/` 与共享库重复实现。
3. **持续探索并记录其他优化项。** 每项改动应说明收益、兼容性影响和验证方式。优先评估：

   重构阶段、决策与待办统一记录在 `docs/easytier-web-refactor.md`。

   - API 类型与错误模型统一，减少 `frontend/src/modules/` 和 `frontend-lib/src/modules/` 的重复请求逻辑；
   - 路由懒加载、请求缓存、轮询/WebSocket 更新、图表和设备列表的渲染性能；
   - `src/db/` 的索引、分页、迁移回滚和配置版本并发一致性；
   - CORS、CSRF、CSP、限流、会话生命周期、审计日志和密钥管理；
   - 后端 tracing、健康检查、超时/重试与运行时配置同步的可观测性；
   - 登录、权限、配置编辑、移动端布局的 Vitest、集成测试和端到端测试覆盖。

## 提交与 Pull Request 指南

使用项目已有的 Conventional Commits 格式，例如 `feat(peer): ...`、`fix(web-client): ...`、`refactor(core): ...` 或 `docs: ...`。每个提交应聚焦于单一目的。Pull Request 应提交到 `develop`，说明变更内容和已执行的验证，关联相关 issue，并在行为或配置发生变化时同步更新文档。涉及 GUI 界面变化时请附上截图。

## 安全与配置提示

不要提交凭据、私有网络密钥、签名密钥或本地生成的构建产物。保持 `Cargo.lock` 一致，并确认 `cargo metadata --locked` 可以通过；提交 PR 前还应检查相关平台的 CI 工作流。
