#![allow(dead_code)]

#[macro_use]
extern crate rust_i18n;

use std::path::PathBuf;
use std::sync::Arc;
use std::{net::IpAddr, time::Duration};

#[cfg(feature = "embed")]
use anyhow::Context as _;
use clap::Parser;
use easytier::tunnel::websocket::WsTunnelListener;
use easytier::{
    common::{
        config::{ConsoleLoggerConfig, FileLoggerConfig, LoggingConfigLoader},
        constants::EASYTIER_VERSION,
        error::Error,
        log,
        network::{local_ipv4, local_ipv6},
    },
    proto::rpc::standalone::{runtime_rpc_listener, runtime_udp_tunnel_listener},
    utils::panic::setup_panic_handler,
};
use easytier_core::{socket::SocketListener, tunnel::Tunnel};

use easytier::tunnel::IpScheme;
use mimalloc::MiMalloc;

mod client_manager;
mod config;
mod db;
mod migrator;
mod restful;
mod webhook;

#[cfg(feature = "embed")]
mod web;

#[global_allocator]
static GLOBAL_MIMALLOC: MiMalloc = MiMalloc;

rust_i18n::i18n!("locales", fallback = "en");

#[derive(Parser, Debug)]
#[command(name = "easytier-web", author, version = EASYTIER_VERSION , about, long_about = None)]
struct Cli {
    #[arg(
        long,
        env = "ET_WEB_CONFIG_FILE",
        help = t!("cli.config_file").to_string()
    )]
    config_file: Option<PathBuf>,

    #[arg(
        short,
        long,
        env = "ET_WEB_DB",
        help = t!("cli.db").to_string()
    )]
    db: Option<String>,

    #[arg(
        long,
        env = "ET_WEB_CONSOLE_LOG_LEVEL",
        help = t!("cli.console_log_level").to_string(),
    )]
    console_log_level: Option<String>,

    #[arg(
        long,
        env = "ET_WEB_FILE_LOG_LEVEL",
        help = t!("cli.file_log_level").to_string(),
    )]
    file_log_level: Option<String>,

    #[arg(
        long,
        env = "ET_WEB_FILE_LOG_DIR",
        help = t!("cli.file_log_dir").to_string(),
    )]
    file_log_dir: Option<String>,

    #[arg(
        long,
        short='c',
        env = "ET_CONFIG_SERVER_PORT",
        help = t!("cli.config_server_port").to_string(),
    )]
    config_server_port: Option<u16>,

    #[arg(
        long,
        short='p',
        env = "ET_CONFIG_SERVER_PROTOCOL",
        help = t!("cli.config_server_protocol").to_string(),
    )]
    config_server_protocol: Option<String>,

    #[arg(
        long,
        short='a',
        env = "ET_API_SERVER_PORT",
        help = t!("cli.api_server_port").to_string(),
    )]
    api_server_port: Option<u16>,

    #[arg(
        long,
        env = "ET_API_SERVER_ADDR",
        help = t!("cli.api_server_addr").to_string(),
    )]
    api_server_addr: Option<IpAddr>,

    #[arg(
        long,
        env = "ET_GEOIP_DB",
        help = t!("cli.geoip_db").to_string(),
    )]
    geoip_db: Option<String>,

    #[arg(
        long,
        env = "ET_HEARTBEAT_MIN_RESPONSE_MS",
        help = t!("cli.heartbeat_min_response_ms").to_string(),
    )]
    heartbeat_min_response_ms: Option<u64>,

    #[cfg(feature = "embed")]
    #[arg(
        long,
        short='l',
        env = "ET_WEB_SERVER_PORT",
        help = t!("cli.web_server_port").to_string(),
    )]
    web_server_port: Option<u16>,

    #[cfg(feature = "embed")]
    #[arg(
        long,
        env = "ET_WEB_SERVER_ADDR",
        help = t!("cli.web_server_addr").to_string(),
    )]
    web_server_addr: Option<IpAddr>,

    #[cfg(feature = "embed")]
    #[arg(
        long,
        env = "ET_NO_WEB",
        help = t!("cli.no_web").to_string(),
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true"
    )]
    no_web: Option<bool>,

    #[cfg(feature = "embed")]
    #[arg(
        long,
        env = "ET_API_HOST",
        help = t!("cli.api_host").to_string()
    )]
    api_host: Option<String>,

    #[command(flatten)]
    feature_flags: FeatureFlagOptions,

    #[command(flatten)]
    oidc: restful::oidc::OidcOptions,

    #[command(flatten)]
    webhook: WebhookOptions,
}

#[derive(Debug, Clone, Default, clap::Args)]
pub struct WebhookOptions {
    /// Base URL of the webhook endpoint for token validation and event delivery.
    /// When set, incoming tokens are validated via this webhook before local fallback.
    #[arg(long, env = "ET_WEBHOOK_URL")]
    pub webhook_url: Option<String>,

    /// Shared secret used to authenticate outbound webhook calls.
    #[arg(long, env = "ET_WEBHOOK_SECRET", hide_env_values = true)]
    pub webhook_secret: Option<String>,

    /// Token for X-Internal-Auth header. When set, API requests with this header
    /// bypass session authentication.
    #[arg(long, env = "ET_INTERNAL_AUTH_TOKEN", hide_env_values = true)]
    pub internal_auth_token: Option<String>,

    /// Stable identifier for this easytier-web instance when routing webhook callbacks.
    #[arg(long, env = "ET_WEB_INSTANCE_ID")]
    pub web_instance_id: Option<String>,

    /// Reachable base URL for this easytier-web instance's internal REST API.
    #[arg(long, env = "ET_WEB_INSTANCE_API_BASE_URL")]
    pub web_instance_api_base_url: Option<String>,
}

#[derive(Debug, Clone, Default, clap::Args)]
struct FeatureFlagOptions {
    /// Whether user registration via the web UI is disabled.
    #[arg(
        long,
        env = "ET_DISABLE_REGISTRATION",
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        help = t!("cli.disable_registration").to_string()
    )]
    disable_registration: Option<bool>,

    /// Whether to auto-create users when they connect via heartbeat with an unknown token.
    #[arg(
        long,
        env = "ET_ALLOW_AUTO_CREATE_USER",
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        help = t!("cli.allow_auto_create_user").to_string()
    )]
    allow_auto_create_user: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct FeatureFlags {
    pub disable_registration: bool,
    pub allow_auto_create_user: bool,
}

struct ResolvedCli {
    db: String,
    console_log_level: Option<String>,
    file_log_level: Option<String>,
    file_log_dir: Option<String>,
    config_server_port: u16,
    config_server_protocol: String,
    api_server_port: u16,
    api_server_addr: IpAddr,
    geoip_db: Option<String>,
    heartbeat_min_response_ms: u64,
    #[cfg(feature = "embed")]
    web_server_port: Option<u16>,
    #[cfg(feature = "embed")]
    web_server_addr: IpAddr,
    #[cfg(feature = "embed")]
    no_web: bool,
    #[cfg(feature = "embed")]
    api_host: Option<url::Url>,
    feature_flags: FeatureFlags,
    oidc: restful::oidc::OidcOptions,
    webhook: WebhookOptions,
}

fn merge_value<T>(cli: Option<T>, file: Option<T>, default: T) -> T {
    cli.or(file).unwrap_or(default)
}

impl Cli {
    fn resolve(self, file: config::WebConfigFile) -> anyhow::Result<ResolvedCli> {
        #[cfg(feature = "embed")]
        let api_host = self
            .api_host
            .or(file.web.api_host)
            .map(|value| value.parse())
            .transpose()
            .context("invalid API host URL")?;

        Ok(ResolvedCli {
            db: merge_value(self.db, file.server.db, "et.db".to_string()),
            console_log_level: self.console_log_level.or(file.server.console_log_level),
            file_log_level: self.file_log_level.or(file.server.file_log_level),
            file_log_dir: self.file_log_dir.or(file.server.file_log_dir),
            config_server_port: merge_value(
                self.config_server_port,
                file.server.config_server_port,
                22020,
            ),
            config_server_protocol: merge_value(
                self.config_server_protocol,
                file.server.config_server_protocol,
                "udp".to_string(),
            ),
            api_server_port: merge_value(self.api_server_port, file.server.api_server_port, 11211),
            api_server_addr: merge_value(
                self.api_server_addr,
                file.server.api_server_addr,
                "0.0.0.0".parse().unwrap(),
            ),
            geoip_db: self.geoip_db.or(file.server.geoip_db),
            heartbeat_min_response_ms: merge_value(
                self.heartbeat_min_response_ms,
                file.server.heartbeat_min_response_ms,
                0,
            ),
            #[cfg(feature = "embed")]
            web_server_port: self.web_server_port.or(file.web.web_server_port),
            #[cfg(feature = "embed")]
            web_server_addr: merge_value(
                self.web_server_addr,
                file.web.web_server_addr,
                "0.0.0.0".parse().unwrap(),
            ),
            #[cfg(feature = "embed")]
            no_web: merge_value(self.no_web, file.web.no_web, false),
            #[cfg(feature = "embed")]
            api_host,
            feature_flags: FeatureFlags {
                disable_registration: merge_value(
                    self.feature_flags.disable_registration,
                    file.features.disable_registration,
                    false,
                ),
                allow_auto_create_user: merge_value(
                    self.feature_flags.allow_auto_create_user,
                    file.features.allow_auto_create_user,
                    false,
                ),
            },
            oidc: restful::oidc::OidcOptions {
                oidc_issuer_url: self.oidc.oidc_issuer_url.or(file.oidc.issuer_url),
                oidc_client_id: self.oidc.oidc_client_id.or(file.oidc.client_id),
                oidc_client_secret: self.oidc.oidc_client_secret.or(file.oidc.client_secret),
                oidc_username_claim: self.oidc.oidc_username_claim.or(file.oidc.username_claim),
                oidc_provider_name: self.oidc.oidc_provider_name.or(file.oidc.provider_name),
                oidc_scopes: self.oidc.oidc_scopes.or(file.oidc.scopes),
                oidc_redirect_url: self.oidc.oidc_redirect_url.or(file.oidc.redirect_url),
                oidc_disable_pkce: self.oidc.oidc_disable_pkce.or(file.oidc.disable_pkce),
                oidc_frontend_base_url: self
                    .oidc
                    .oidc_frontend_base_url
                    .or(file.oidc.frontend_base_url),
            },
            webhook: WebhookOptions {
                webhook_url: self.webhook.webhook_url.or(file.webhook.url),
                webhook_secret: self.webhook.webhook_secret.or(file.webhook.secret),
                internal_auth_token: self
                    .webhook
                    .internal_auth_token
                    .or(file.webhook.internal_auth_token),
                web_instance_id: self.webhook.web_instance_id.or(file.webhook.instance_id),
                web_instance_api_base_url: self
                    .webhook
                    .web_instance_api_base_url
                    .or(file.webhook.instance_api_base_url),
            },
        })
    }
}

impl LoggingConfigLoader for &ResolvedCli {
    fn get_console_logger_config(&self) -> ConsoleLoggerConfig {
        ConsoleLoggerConfig {
            level: self.console_log_level.clone(),
        }
    }

    fn get_file_logger_config(&self) -> FileLoggerConfig {
        FileLoggerConfig {
            dir: self.file_log_dir.clone(),
            level: self.file_log_level.clone(),
            file: None,
            size_mb: None,
            count: None,
        }
    }
}

pub fn get_listener_by_url(
    scheme: IpScheme,
    l: &url::Url,
) -> Option<Box<dyn SocketListener<Accepted = Box<dyn Tunnel>>>> {
    Some(match scheme {
        IpScheme::Tcp => {
            let addr = l.socket_addrs(|| Some(11010)).ok()?.into_iter().next()?;
            Box::new(runtime_rpc_listener(addr))
        }
        IpScheme::Udp => {
            let addr = l.socket_addrs(|| Some(11010)).ok()?.into_iter().next()?;
            Box::new(runtime_udp_tunnel_listener(l.clone(), addr))
        }
        IpScheme::Ws => Box::new(WsTunnelListener::new(l.clone())),
        _ => return None,
    })
}

async fn get_dual_stack_listener(
    protocol: &str,
    port: u16,
) -> Result<
    (
        Option<Box<dyn SocketListener<Accepted = Box<dyn Tunnel>>>>,
        Option<Box<dyn SocketListener<Accepted = Box<dyn Tunnel>>>>,
    ),
    Error,
> {
    let scheme = protocol
        .parse()
        .map_err(|_| Error::InvalidUrl(protocol.to_string()))?;
    let v6_listener =
        if local_ipv6().await.is_ok() && matches!(scheme, IpScheme::Tcp | IpScheme::Udp) {
            get_listener_by_url(
                scheme,
                &format!("{protocol}://[::]:{port}").parse().unwrap(),
            )
        } else {
            None
        };
    let v4_listener = if local_ipv4().await.is_ok() {
        get_listener_by_url(
            scheme,
            &format!("{protocol}://0.0.0.0:{port}").parse().unwrap(),
        )
    } else {
        None
    };
    Ok((v6_listener, v4_listener))
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    rust_i18n::set_locale(&locale);
    setup_panic_handler();

    let cli = Cli::parse();
    let config_file = match config::WebConfigFile::load(cli.config_file.as_deref()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load web configuration: {e:#}");
            std::process::exit(2);
        }
    };
    let cli = match cli.resolve(config_file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to resolve web configuration: {e:#}");
            std::process::exit(2);
        }
    };
    log::init(&cli, false).unwrap();

    // Validate OIDC configuration: check split-deploy specific requirements
    // Basic OIDC parameter validation is handled in OidcConfig::from_params
    if cli.oidc.any_param_provided() {
        let is_split_deploy = {
            #[cfg(feature = "embed")]
            {
                let embed_split_by_port = cli.web_server_port.is_some()
                    && cli.web_server_port != Some(cli.api_server_port);
                cli.no_web || embed_split_by_port
            }
            #[cfg(not(feature = "embed"))]
            {
                true
            }
        };

        if is_split_deploy && cli.oidc.oidc_frontend_base_url.is_none() {
            eprintln!("Error: --oidc-frontend-base-url is required in split-deploy mode");
            eprintln!(
                "When frontend and API are deployed separately, you must specify the frontend URL"
            );
            eprintln!("Example: --oidc-frontend-base-url http://your-frontend-domain.com");
            std::process::exit(1);
        }
    }

    // let db = db::Db::new(":memory:").await.unwrap();
    let db = db::Db::new(cli.db).await.unwrap();
    let feature_flags = Arc::new(cli.feature_flags);
    let webhook_config = Arc::new(webhook::WebhookConfig::new(
        cli.webhook.webhook_url,
        cli.webhook.webhook_secret,
        cli.webhook.internal_auth_token,
        cli.webhook.web_instance_id,
        cli.webhook.web_instance_api_base_url,
    ));
    let mut mgr = client_manager::ClientManager::new(
        db.clone(),
        cli.geoip_db,
        Duration::from_millis(cli.heartbeat_min_response_ms),
        feature_flags.clone(),
        webhook_config.clone(),
    );
    let (v6_listener, v4_listener) =
        get_dual_stack_listener(&cli.config_server_protocol, cli.config_server_port)
            .await
            .unwrap();
    if v4_listener.is_none() && v6_listener.is_none() {
        panic!("Listen to both IPv4 and IPv6 failed");
    }
    if let Some(listener) = v6_listener {
        mgr.add_listener(listener).await.unwrap();
    }
    if let Some(listener) = v4_listener {
        mgr.add_listener(listener).await.unwrap();
    }

    let mgr = Arc::new(mgr);

    #[cfg(feature = "embed")]
    let (web_router_restful, web_router_static) = if cli.no_web {
        (None, None)
    } else {
        let web_router = web::build_router(cli.api_host.clone());
        if cli.web_server_port.is_none()
            || (cli.web_server_port == Some(cli.api_server_port)
                && cli.web_server_addr == cli.api_server_addr)
        {
            (Some(web_router), None)
        } else {
            (None, Some(web_router))
        }
    };
    #[cfg(not(feature = "embed"))]
    let web_router_restful = None;

    let oidc_config = if cli.oidc.oidc_issuer_url.is_some() {
        match restful::oidc::OidcConfig::from_params(cli.oidc).await {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to initialize OIDC: {:?}", e);
                eprintln!("Please check your OIDC configuration (issuer URL, client ID, etc.)");
                std::process::exit(1);
            }
        }
    } else {
        restful::oidc::OidcConfig::disabled()
    };

    let _restful_server_tasks = restful::RestfulServer::new(
        std::net::SocketAddr::new(cli.api_server_addr, cli.api_server_port),
        mgr.clone(),
        db,
        web_router_restful,
        feature_flags,
        oidc_config,
        webhook_config,
    )
    .await
    .unwrap()
    .start()
    .await
    .unwrap();

    #[cfg(feature = "embed")]
    let _web_server_task = if let Some(web_router) = web_router_static {
        Some(
            web::WebServer::new(
                std::net::SocketAddr::new(cli.web_server_addr, cli.web_server_port.unwrap_or(0)),
                web_router,
            )
            .await
            .unwrap()
            .start()
            .await
            .unwrap(),
        )
    } else {
        None
    };

    tokio::signal::ctrl_c().await.unwrap();
}
