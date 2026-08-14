# easytier-web TOML 配置

`easytier-web` 支持通过 `--config-file <PATH>` 或 `ET_WEB_CONFIG_FILE` 指定一个 TOML 启动配置文件。

```bash
./easytier-web --config-file /etc/easytier/web.toml
```

配置文件只在进程启动时读取一次。修改文件后需要重启 `easytier-web`，当前不支持运行时热加载。

## 配置优先级

同一选项同时出现在多个来源时，优先级从高到低如下：

1. 命令行参数
2. 环境变量
3. TOML 配置文件
4. 内置默认值

`--config-file` 和 `ET_WEB_CONFIG_FILE` 只用于指定配置文件位置，不属于 TOML 文件内容。配置文件使用未知字段会导致启动失败，以便及时发现拼写错误。

## 完整示例

下面的配置覆盖所有可通过 TOML 设置的 `easytier-web` 启动选项。没有需要设置的选项可以删除；未填写的选项使用默认值。

```toml
[server]
db = "/var/lib/easytier/et.db"
console_log_level = "info"
file_log_level = "info"
file_log_dir = "/var/log/easytier"
config_server_port = 22020
config_server_protocol = "udp"
api_server_port = 11211
api_server_addr = "0.0.0.0"
geoip_db = "/var/lib/easytier/GeoLite.mmdb"
heartbeat_min_response_ms = 0

[web]
# 仅 embed 构建使用。
web_server_port = 11211
web_server_addr = "0.0.0.0"
no_web = false
api_host = "https://easytier.example.com"

[features]
disable_registration = false
allow_auto_create_user = false

[oidc]
issuer_url = "https://auth.example.com"
client_id = "easytier-web"
# 生产环境建议使用 Secret Manager 或受保护的文件权限。
client_secret = "replace-with-client-secret"
username_claim = "preferred_username"
provider_name = "Authelia"
scopes = ["openid", "profile"]
redirect_url = "https://easytier.example.com/api/v1/auth/oidc/callback"
disable_pkce = false
# 前后端分离时必须设置。
frontend_base_url = "https://console.example.com"

[webhook]
url = "https://webhook.example.com"
secret = "replace-with-webhook-secret"
internal_auth_token = "replace-with-internal-token"
instance_id = "web-1"
instance_api_base_url = "https://api.example.com"
```

## 字段映射

### `[server]`

| TOML 字段 | CLI 参数 | 环境变量 | 默认值 |
| --- | --- | --- | --- |
| `db` | `--db` | `ET_WEB_DB` | `et.db` |
| `console_log_level` | `--console-log-level` | `ET_WEB_CONSOLE_LOG_LEVEL` | 无 |
| `file_log_level` | `--file-log-level` | `ET_WEB_FILE_LOG_LEVEL` | 无 |
| `file_log_dir` | `--file-log-dir` | `ET_WEB_FILE_LOG_DIR` | 无 |
| `config_server_port` | `--config-server-port` | `ET_CONFIG_SERVER_PORT` | `22020` |
| `config_server_protocol` | `--config-server-protocol` | `ET_CONFIG_SERVER_PROTOCOL` | `udp` |
| `api_server_port` | `--api-server-port` | `ET_API_SERVER_PORT` | `11211` |
| `api_server_addr` | `--api-server-addr` | `ET_API_SERVER_ADDR` | `0.0.0.0` |
| `geoip_db` | `--geoip-db` | `ET_GEOIP_DB` | 无 |
| `heartbeat_min_response_ms` | `--heartbeat-min-response-ms` | `ET_HEARTBEAT_MIN_RESPONSE_MS` | `0` |

### `[web]`

| TOML 字段 | CLI 参数 | 环境变量 | 默认值 |
| --- | --- | --- | --- |
| `web_server_port` | `--web-server-port` | `ET_WEB_SERVER_PORT` | 无 |
| `web_server_addr` | `--web-server-addr` | `ET_WEB_SERVER_ADDR` | `0.0.0.0` |
| `no_web` | `--no-web` | `ET_NO_WEB` | `false` |
| `api_host` | `--api-host` | `ET_API_HOST` | 无 |

`[web]` 中的字段在非 `embed` 构建中仍可被解析，但嵌入式静态 Web 服务相关字段不会生效。

### `[features]`

| TOML 字段 | CLI 参数 | 环境变量 | 默认值 |
| --- | --- | --- | --- |
| `disable_registration` | `--disable-registration` | `ET_DISABLE_REGISTRATION` | `false` |
| `allow_auto_create_user` | `--allow-auto-create-user` | `ET_ALLOW_AUTO_CREATE_USER` | `false` |

### `[oidc]`

| TOML 字段 | CLI 参数 | 环境变量 | 默认值 |
| --- | --- | --- | --- |
| `issuer_url` | `--oidc-issuer-url` | `ET_OIDC_ISSUER_URL` | 无 |
| `client_id` | `--oidc-client-id` | `ET_OIDC_CLIENT_ID` | 无 |
| `client_secret` | `--oidc-client-secret` | `OIDC_CLIENT_SECRET` | 无 |
| `username_claim` | `--oidc-username-claim` | `ET_OIDC_USERNAME_CLAIM` | `preferred_username` |
| `provider_name` | `--oidc-provider-name` | `ET_OIDC_PROVIDER_NAME` | `Single Sign-On` |
| `scopes` | `--oidc-scopes` | 无 | `["openid", "profile"]` |
| `redirect_url` | `--oidc-redirect-url` | `ET_OIDC_REDIRECT_URL` | 无 |
| `disable_pkce` | `--oidc-disable-pkce` | 无 | `false` |
| `frontend_base_url` | `--oidc-frontend-base-url` | `ET_OIDC_FRONTEND_BASE_URL` | 无 |

设置 `issuer_url` 后，`client_id` 和 `redirect_url` 也必须设置。`scopes` 是 TOML 字符串数组，程序会自动确保包含 `openid`。分离部署时必须设置 `frontend_base_url`。OIDC 的 `client_secret` 不会通过公开 API 返回。

### `[webhook]`

| TOML 字段 | CLI 参数 | 环境变量 | 默认值 |
| --- | --- | --- | --- |
| `url` | `--webhook-url` | `ET_WEBHOOK_URL` | 无 |
| `secret` | `--webhook-secret` | `ET_WEBHOOK_SECRET` | 无 |
| `internal_auth_token` | `--internal-auth-token` | `ET_INTERNAL_AUTH_TOKEN` | 无 |
| `instance_id` | `--web-instance-id` | `ET_WEB_INSTANCE_ID` | 无 |
| `instance_api_base_url` | `--web-instance-api-base-url` | `ET_WEB_INSTANCE_API_BASE_URL` | 无 |

## 实现方式

实现位于 `easytier-web/src/config.rs` 和 `easytier-web/src/main.rs`：

1. `--config-file` 或 `ET_WEB_CONFIG_FILE` 指定文件路径。
2. `config.rs` 使用 `serde` 和 `toml` 将五个 TOML 分组反序列化为可选字段。
3. `main.rs` 将命令行、环境变量、文件值按优先级逐字段合并。
4. 合并完成后再应用默认值，并执行 API 地址、OIDC 参数和监听参数的现有校验。
5. OIDC Discovery 仍在启动阶段执行，配置文件不会改变现有认证流程或 REST API。

采用可选字段和显式合并，而不是给 Clap 参数直接设置默认值，是为了区分“未设置”和“显式设置默认值”，从而保证命令行/环境变量能够可靠覆盖 TOML 配置。

## 安全建议

配置文件可能包含 OIDC Client Secret、Webhook Secret 和内部认证 Token。生产环境应限制文件权限，例如 `chmod 600 /etc/easytier/web.toml`，并优先使用 Secret Manager、容器 Secret 或环境变量注入敏感值。不要将包含真实凭据的配置文件提交到 Git。
