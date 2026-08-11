# easytier-web 构建与产物发布策略

## 目标与原则

`easytier-web` 采用“PR 构建一次、main 校验并提升、无法复用时再构建”的策略。源码必须先提交到远端分支；编译生成物不得提交到 Git，而应存放在 GitHub Actions Artifacts 或 Release。默认只为独立 Web 版本构建 Linux `x86_64-unknown-linux-gnu`，避免因页面或 OAuth 变更启动无关平台矩阵。

## 触发与职责

| 事件 | 行为 | 产物用途 |
| --- | --- | --- |
| PR 修改 `easytier-web/**` | 构建前端，运行 Rust/OIDC 测试，编译 Linux release，生成校验清单 | 合并候选产物 |
| push 到 `main` | 定位已合并 PR，验证构建输入一致性，下载并重新上传候选产物 | main 可追溯产物 |
| 无可用或可信候选产物 | 在 main 仅重新构建 Linux | 回退产物 |
| easytier-web beta 标签 | 发布已验证的 main 产物并标记 prerelease | beta Release |
| EasyTier 正式版本标签 | 按正式发布策略运行所需平台矩阵 | 正式 Release |

## 构建输入与清单

Squash 合并会改变提交 SHA，因此不能仅比较 PR SHA 与 main SHA。PR 构建应计算并记录实际构建输入摘要，至少覆盖：

- `easytier-web/**`；
- `Cargo.toml`、`Cargo.lock`；
- `package.json`、`pnpm-lock.yaml`、`pnpm-workspace.yaml`；
- 被 Web 包引用的共享源码及 `.github/actions/**`。

Artifact 内必须附带 `manifest.json`，记录源提交、Git tree、输入摘要、目标平台、Rust/Node/pnpm 版本、工作流运行 ID、二进制 SHA-256 和构建时间。二进制上传前还应使用 `file` 验证为目标 ELF。

## main 产物提升流程

1. 从 main push 对应的合并记录解析 PR 编号和 PR head SHA。
2. 查找该 PR 最新且成功的 Linux 构建运行。
3. 仅接受同仓库受信分支产生、未过期且结论为成功的 Artifact。
4. 重新计算 main 的构建输入摘要，并与 `manifest.json` 比较。
5. 校验 Artifact 摘要和二进制 SHA-256；全部一致后重新上传到 main 运行。
6. 任一检查失败时，不复用旧产物，自动执行 Linux-only 回退构建。

提升后的名称应包含版本、目标和 main 短 SHA，例如 `easytier-web-linux-x86_64-b566c730`。工作流 Summary 应记录来源 PR、原始运行、main SHA、摘要和回退原因。注意 Artifact 绑定到原始 workflow run；“重复构建跳过”不会自动复制产物，提升步骤必须显式下载并重新上传。

## 安全与发布门禁

- PR 工作流使用最小 `contents: read` 权限，不向不可信代码暴露 Release 密钥。
- main 提升工作流可读取 Actions，但只有标签发布任务需要 `contents: write`。
- beta 发布前必须确认测试、Linux release 构建、ELF 检查和摘要验证全部通过。
- Artifact 过期、来源不明、输入不一致或校验失败时必须回退重建，不允许强制提升。
- Release 必须标明源 main SHA、工作流链接、目标平台和 SHA-256。

## 实施顺序

先让 Linux PR 工作流生成标准清单，再新增 main 产物提升工作流；验证稳定后，将 easytier-web-only 变更从现有 Core/GUI 多平台路径中分离。最后调整 Release 工作流，使其接收已提升的 main 运行 ID，并保留手动 Linux 回退入口。
