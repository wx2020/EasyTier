# easytier-web 重构记录

## 第一阶段：认证基础与登录体验

本阶段保持 `/api/v1/auth/oidc/*` 路径兼容，将 OAuth 2.0/OIDC 登录整理为可配置的产品能力：

- 使用 OIDC 授权码流程、PKCE、state 和 nonce 校验；
- 以 `(issuer, subject)` 绑定外部身份，不再按用户名直接关联本地账户；
- 登录配置接口返回提供商名称、授权流程和 PKCE 状态；
- 登录页显示提供商状态，提供响应式布局、明确反馈和无障碍标签；
- 页面路由改为动态导入，降低初始 JavaScript 负载。

## Authelia 配置示例

在 Authelia 中注册一个 OIDC 客户端，启用 authorization code 与 PKCE `S256`，并将回调地址精确设置为 EasyTier 的回调地址。启动时可使用环境变量或 [TOML 配置文件](easytier-web-config.md)：

```bash
export ET_OIDC_ISSUER_URL="https://<authelia-issuer>"
export ET_OIDC_CLIENT_ID="easytier-web"
export OIDC_CLIENT_SECRET="<client-secret>"
export ET_OIDC_REDIRECT_URL="https://<easytier-host>/api/v1/auth/oidc/callback"
export ET_OIDC_PROVIDER_NAME="Authelia"
cargo run -p easytier-web
```

默认请求 `openid,profile` scopes，并从 `preferred_username` 读取本地显示名。分离部署时还需设置 `ET_OIDC_FRONTEND_BASE_URL`。客户端密钥只能通过受保护的运行环境或受限权限的配置文件提供，不得写入仓库或前端代码。

旧版本只保存 OIDC 用户名，没有保存 `issuer` 和 `subject`，因此无法安全地自动迁移身份绑定。升级后首次登录会创建新的外部身份记录；若首选用户名已被本地账户占用，则创建带 `~oidc-<随机后缀>` 的隔离账户，管理员可在确认身份后通过后续账户关联功能处理数据迁移。

## TOML 启动配置记录

- 使用 `--config-file <PATH>` 或 `ET_WEB_CONFIG_FILE` 指定 `easytier-web` 的 TOML 启动配置。
- 配置分为 `[server]`、`[web]`、`[features]`、`[oidc]` 和 `[webhook]` 五个分组，字段映射见 [easytier-web TOML 配置](easytier-web-config.md)。
- 配置优先级固定为：命令行参数 > 环境变量 > TOML 配置 > 内置默认值。
- 配置文件只在启动时读取，不支持热加载；未知字段启动失败。
- OIDC Client Secret、Webhook Secret 和内部 Token 属于敏感信息，配置文件必须使用受限权限，生产环境优先使用 Secret Manager、容器 Secret 或环境变量注入。
- 格式化、编译和测试由远端执行；本地只做源码格式化和静态差异检查。

## 后续优化清单

1. **P0—认证安全：** 增加显式账户关联/解绑、角色映射、统一退出和登录审计；评估会话 Cookie、CSP、CORS 与限流策略。
2. **P1—API 一致性：** 统一 REST 错误结构和前端请求封装，生成稳定类型，减少两个 `modules/api.ts` 的重复逻辑。
3. **P1—性能：** 为设备列表增加分页/虚拟滚动，合并高频轮询，评估事件流或 WebSocket，并为关键查询补索引。
4. **P1—质量：** 增加 Authelia 登录端到端测试、权限测试、移动端视口测试和迁移回滚测试。
5. **P2—可观测性：** 为认证、设备会话和配置同步增加指标、关联 ID、健康检查与可诊断的超时信息。
