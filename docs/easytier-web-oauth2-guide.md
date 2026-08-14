# easytier-web OAuth 2.0/OIDC 接入与执行指南

## 实现范围

`easytier-web` 使用 OAuth 2.0 **Authorization Code** 流程完成授权，并使用 OpenID Connect（OIDC）发现、ID Token、`issuer`、`subject` 和 `nonce` 完成身份认证。因此身份提供商必须支持 OIDC，不能只提供不含 ID Token 的纯 OAuth 2.0 接口。Authelia、Keycloak 等标准 OIDC 提供商均可接入。

当前实现包括：

- OIDC Discovery 自动发现授权端点、Token 端点和签名密钥；
- 默认启用 PKCE `S256`；
- 使用随机 `state` 防止 CSRF，并以常量时间比较回调值；
- 使用随机 `nonce` 校验 ID Token，验证签名、issuer、audience、有效期及可选 `at_hash`；
- 以稳定的 `(issuer, subject)` 绑定本地用户；
- 登录成功后创建 EasyTier Web 会话，并轮换 Session ID。

## 身份提供商注册

在身份提供商中创建 OIDC 客户端时，应使用以下设置：

| 项目 | 推荐值 |
| --- | --- |
| Client ID | `easytier-web` |
| Grant type | `authorization_code` |
| Response type | `code` |
| Redirect URI | `https://easytier.example.com/api/v1/auth/oidc/callback` |
| PKCE | `S256`，必需或允许 |
| Scopes | `openid profile` |
| Username claim | `preferred_username` |

Redirect URI 必须与 EasyTier 配置及身份提供商登记值完全一致，包括协议、域名、端口、路径和末尾斜杠。机密客户端应生成 Client Secret；公共客户端可以不设置 Secret，但身份提供商必须允许 Authorization Code + PKCE。

以 Authelia 为例，应注册一个 OIDC 客户端，启用授权码流程、PKCE `S256`、`openid`/`profile` scopes，并确保 ID Token 中包含稳定的 `sub` 和可用的 `preferred_username`。Authelia 的配置语法应以部署版本的官方文档为准。

## Authelia 建议配置

下面的片段只包含 easytier-web 客户端。Authelia 还必须完成 OIDC Provider、存储和 JWKS 签名密钥等全局配置，参见 [Authelia OIDC Provider 配置](https://www.authelia.com/configuration/identity-providers/openid-connect/provider/)。

先生成随机客户端 Secret 及其 PBKDF2-SHA512 摘要：

```bash
docker run --rm authelia/authelia:latest \
  authelia crypto hash generate pbkdf2 \
  --variant sha512 --random --random.length 72 --random.charset rfc3986
```

命令会输出明文随机密码和 `Digest`。明文只提供给 easytier-web，摘要写入 Authelia；不要把明文写入 Authelia 配置或 Git。该生成方式来自 [Authelia 客户端凭据指南](https://www.authelia.com/integration/openid-connect/frequently-asked-questions/#client-secret)。

将以下内容合并到 Authelia 的 `configuration.yml`，替换域名和摘要占位符：

```yaml
identity_providers:
  oidc:
    # 保留现有 OIDC Provider、JWKS 等配置。
    clients:
      - client_id: easytier-web
        client_name: EasyTier Web
        client_secret: '<生成的 Digest>'
        public: false
        authorization_policy: two_factor
        require_pkce: true
        pkce_challenge_method: S256
        redirect_uris:
          - https://easytier.example.com/api/v1/auth/oidc/callback
        scopes:
          - openid
          - profile
        response_types:
          - code
        grant_types:
          - authorization_code
        token_endpoint_auth_method: client_secret_basic
```

这组配置与 easytier-web 当前后端行为匹配：[Authelia 客户端配置参考](https://www.authelia.com/configuration/identity-providers/openid-connect/clients/)要求回调 URI 精确匹配，并说明机密客户端默认使用 `client_secret_basic`。`authorization_policy: two_factor` 是生产建议；仅在明确接受单因素风险时才改为 `one_factor`。

对应的 easytier-web 环境变量如下，其中 `OIDC_CLIENT_SECRET` 必须使用生成命令输出的**明文随机密码**，不是 Digest：

```bash
export ET_OIDC_ISSUER_URL="https://auth.example.com"
export ET_OIDC_CLIENT_ID="easytier-web"
export OIDC_CLIENT_SECRET="<生成的明文随机密码>"
export ET_OIDC_REDIRECT_URL="https://easytier.example.com/api/v1/auth/oidc/callback"
export ET_OIDC_PROVIDER_NAME="Authelia"
export ET_OIDC_USERNAME_CLAIM="preferred_username"
```

应用 Authelia 配置前先执行其配置校验；重启后确认 `https://auth.example.com/.well-known/openid-configuration` 可从 easytier-web 容器或主机访问。若 easytier-web 与 Authelia 位于不同容器网络，`ET_OIDC_ISSUER_URL` 仍应使用浏览器可识别且与 Authelia Discovery 中 `issuer` 完全一致的外部 HTTPS 地址。

## easytier-web 配置

### TOML 配置

OIDC 也可以放在 easytier-web 的 TOML 启动配置中：

```toml
[oidc]
issuer_url = "https://auth.example.com"
client_id = "easytier-web"
client_secret = "<client-secret>"
redirect_url = "https://easytier.example.com/api/v1/auth/oidc/callback"
provider_name = "Authelia"
username_claim = "preferred_username"
scopes = ["openid", "profile"]
```

通过 `./easytier-web --config-file /etc/easytier/web.toml` 启动。完整字段、优先级和敏感信息处理见 [easytier-web TOML 配置](easytier-web-config.md)。

### 环境变量

```bash
export ET_OIDC_ISSUER_URL="https://auth.example.com"
export ET_OIDC_CLIENT_ID="easytier-web"
export OIDC_CLIENT_SECRET="<client-secret>"
export ET_OIDC_REDIRECT_URL="https://easytier.example.com/api/v1/auth/oidc/callback"
export ET_OIDC_PROVIDER_NAME="Authelia"
export ET_OIDC_USERNAME_CLAIM="preferred_username"

./easytier-web --db et.db
```

不要把 Client Secret 写入镜像、仓库或前端变量。优先通过 Secret Manager、容器 Secret 或受保护的进程环境注入。

### 参数说明

| CLI 参数 | 环境变量 | 必需/默认值 | 说明 |
| --- | --- | --- | --- |
| `--oidc-issuer-url` | `ET_OIDC_ISSUER_URL` | 启用 OIDC 时必需 | OIDC issuer，用于 Discovery |
| `--oidc-client-id` | `ET_OIDC_CLIENT_ID` | 启用 OIDC 时必需 | 客户端 ID |
| `--oidc-client-secret` | `OIDC_CLIENT_SECRET` | 可选 | 机密客户端的 Secret |
| `--oidc-redirect-url` | `ET_OIDC_REDIRECT_URL` | 启用 OIDC 时必需 | 回调地址 |
| `--oidc-provider-name` | `ET_OIDC_PROVIDER_NAME` | `Single Sign-On` | 登录页显示名称 |
| `--oidc-username-claim` | `ET_OIDC_USERNAME_CLAIM` | `preferred_username` | 用作本地显示名的 claim 路径 |
| `--oidc-scopes` | 无 | `openid,profile` | 逗号分隔或重复传入；始终自动补充 `openid` |
| `--oidc-disable-pkce` | 无 | `false` | 禁用 PKCE，仅用于兼容不支持 PKCE 的提供商 |
| `--oidc-frontend-base-url` | `ET_OIDC_FRONTEND_BASE_URL` | 分离部署时必需 | 回调成功后跳转的前端入口 |

只要显式设置 OIDC 相关参数，就必须同时提供 issuer、Client ID 和 Redirect URI，否则程序启动失败。程序启动时会发起 Discovery 请求，HTTP 超时为 30 秒；issuer 无法访问、TLS 错误或元数据无效都会阻止启动。

`--oidc-username-claim` 支持点路径，例如 `user.name`；数组也可用数字索引，如 `realm_access.roles.0`。目标值必须是非空字符串。

## 同域与分离部署

嵌入式前端与 API 使用相同地址时，登录成功后回调默认跳转到 `/`。以下情况属于分离部署，必须配置 `ET_OIDC_FRONTEND_BASE_URL`：

- 非 `embed` 构建；
- 启用 `--no-web`；
- Web 与 API 使用不同端口。

示例：

```bash
export ET_OIDC_FRONTEND_BASE_URL="https://console.example.com"
export ET_OIDC_REDIRECT_URL="https://api.example.com/api/v1/auth/oidc/callback"
```

前端请求会携带 Cookie，并通过 API Host 动态访问 `/api/v1`。生产环境优先让反向代理把前端和 API 暴露在同一站点下，减少浏览器跨站 Cookie 限制带来的登录失败。

## 登录执行时序

1. 登录页请求 `GET /api/v1/auth/oidc/config`。
2. 若返回 `enabled: true`，页面显示“使用提供商继续”按钮。
3. 浏览器访问 `GET /api/v1/auth/oidc/login`。
4. 后端生成 `state`、`nonce` 和 PKCE verifier/challenge，将敏感临时值写入服务端 Session。
5. 后端以 HTTP 307 将浏览器重定向到身份提供商授权端点。
6. 身份提供商认证用户，并把 `code` 与 `state` 回调到 `/api/v1/auth/oidc/callback`。
7. 后端校验 `state`，删除一次性 Session 数据，并用授权码及 PKCE verifier 换取 Token。
8. 后端校验 ID Token 和 `nonce`，读取 `issuer`、`subject` 及用户名 claim。
9. 后端查找或创建本地用户，建立登录会话并轮换 Session ID。
10. 浏览器被 HTTP 307 重定向到 `/` 或 `ET_OIDC_FRONTEND_BASE_URL`。

任何回调错误、缺少 code/state、state 不匹配、Session 过期、Token 交换失败或 ID Token 校验失败都会终止登录，不会建立本地会话。

## API

### 查询公开配置

```http
GET /api/v1/auth/oidc/config
```

```json
{
  "enabled": true,
  "provider_name": "Authelia",
  "authorization_flow": "authorization_code",
  "pkce_enabled": true
}
```

该接口不会返回 issuer、Client ID 或 Client Secret。未配置 OIDC 时仍返回 HTTP 200，但 `enabled` 为 `false`。

### 开始登录与处理回调

```text
GET /api/v1/auth/oidc/login
GET /api/v1/auth/oidc/callback?code=...&state=...
```

这两个接口应由浏览器导航访问，不应由前端 AJAX 代替重定向。OIDC 未启用时返回 HTTP 400；认证成功后返回临时重定向。

## 用户创建与身份绑定

数据库迁移会自动创建 `external_identities` 表，并为 `(issuer, subject)` 建立唯一索引。首次登录时：

1. 使用 `(issuer, subject)` 查找已有绑定；
2. 未找到时创建随机密码的本地用户，并加入 `users` 组；
3. 如果用户名 claim 已被本地用户占用，创建 `<用户名>~oidc-<8位随机后缀>`；
4. 在同一数据库事务中保存外部身份绑定。

系统不会仅凭相同用户名把 OIDC 身份绑定到已有本地账户，从而避免外部用户接管同名管理员账户。同一 subject 在不同 issuer 下视为不同身份。删除本地用户时，其外部身份记录会级联删除。

旧版本只保存 OIDC 用户名，无法安全推断 issuer/subject，因此升级后的首次登录可能创建新的隔离账户；当前实现没有自动账户合并功能。

## 会话与当前安全注意事项

Web 会话保存在 SQLite，闲置有效期为一天，Cookie 使用签名和 `SameSite=Lax`。服务重启时会重新生成 Cookie 签名密钥，已有浏览器会话可能失效。

当前代码仍有以下生产限制：

- Session Cookie 设置为 `Secure=false`；必须通过 HTTPS 反向代理暴露服务，并应尽快改为可配置的 Secure Cookie。
- CORS 当前使用宽松策略；应限制为实际前端 Origin。
- 尚未提供 OIDC RP-Initiated Logout、显式账户绑定/解绑、角色或组 claim 映射及登录审计页面。
- PKCE 默认开启，除非提供商明确不兼容，否则不要使用 `--oidc-disable-pkce`。

## 验证与排障

先检查公开配置：

```bash
curl -sS https://easytier.example.com/api/v1/auth/oidc/config
```

再确认 issuer 的 Discovery 地址可从 easytier-web 运行环境访问，并核对：

- Redirect URI 是否完全一致；
- Client ID/Secret 是否对应同一个客户端；
- Token 是否包含 `openid` scope、ID Token、`sub` 和用户名 claim；
- 浏览器是否在 login 与 callback 间保留同一个 Session Cookie；
- 反向代理是否正确传递 HTTPS、Host、Cookie 和查询参数；
- 分离部署是否设置 `ET_OIDC_FRONTEND_BASE_URL`。

可用 SQLite 检查绑定结果：

```sql
SELECT issuer, subject, user_id, create_time
FROM external_identities;
```

常见错误对应关系：

| 错误 | 主要原因 |
| --- | --- |
| `Missing or invalid CSRF token in session` | Cookie 丢失、会话过期或访问了不同 API 实例 |
| `CSRF state mismatch` | 回调 state 不一致或登录流程被重复使用 |
| `PKCE ... verifier is missing` | Session 丢失或回调未到达发起登录的实例 |
| `Token exchange failed` | Secret、Redirect URI、授权码或 Token 端点配置错误 |
| `ID token verification failed` | issuer/audience/nonce/签名/有效期不匹配 |
| `Could not extract username` | 配置的 username claim 不存在、为空或不是字符串 |

多副本部署时，SQLite Session 与 OIDC 临时状态必须对所有回调可见，或由负载均衡器保证 login 与 callback 落到同一实例；否则 state、nonce 或 PKCE 校验会失败。
