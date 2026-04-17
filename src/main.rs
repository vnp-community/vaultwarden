#![cfg_attr(feature = "unstable", feature(ip))]
// The recursion_limit is mainly triggered by the json!() macro.
// The more key/value pairs there are the more recursion occurs.
// We want to keep this as low as possible!
#![recursion_limit = "165"]

// When enabled use MiMalloc as malloc instead of the default malloc
#[cfg(feature = "enable_mimalloc")]
use mimalloc::MiMalloc;
#[cfg(feature = "enable_mimalloc")]
#[cfg_attr(feature = "enable_mimalloc", global_allocator)]
static GLOBAL: MiMalloc = MiMalloc;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel_derive_newtype;

use std::{
    collections::HashMap,
    fs::{canonicalize, create_dir_all},
    panic,
    path::Path,
    process::exit,
    str::FromStr,
    thread,
};

use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

#[cfg(unix)]
use tokio::signal::unix::SignalKind;

#[macro_use]
mod error;
mod api;
mod app_state;
mod auth;
pub mod access_control;
#[cfg(test)]
mod tests;
mod config;
mod crypto;
#[macro_use]
mod db;
mod cache;
mod http_client;
mod mail;
mod ratelimit;
mod sso;
mod sso_client;
mod util;
mod webhook_delivery;
mod audit;
mod siem;
pub mod metrics;
pub mod pam;
pub mod tenant;
pub mod device_trust;
pub mod ldap;
pub mod backup;
pub mod mdm;
pub mod alerting;

use crate::api::core::two_factor::duo_oidc::purge_duo_contexts;
use crate::api::purge_auth_requests;
use crate::api::{WS_ANONYMOUS_SUBSCRIPTIONS, WS_USERS};
pub use config::{PathType, CONFIG};
pub use error::{Error, MapResult};
use rocket::data::{Limits, ToByteUnit};
use std::sync::{atomic::Ordering, Arc};
pub use util::is_running_in_container;

#[rocket::main]
async fn main() -> Result<(), Error> {
    parse_args();
    launch_info();

    let level = init_logging()?;

    // SEC-MED-04: Audit config.json for secrets and file permissions at startup
    config::audit_config_file_for_secrets();
    config::check_config_file_permissions();
    // SEC-LOW-04-A: Warn if backup folder is inside the data folder (web-accessible risk)
    config::check_backup_location();

    check_data_folder().await;
    auth::initialize_keys().await.unwrap_or_else(|e| {
        error!("Error creating private key '{}'\n{e:?}\nExiting Vaultwarden!", CONFIG.private_rsa_key());
        exit(1);
    });

    // SEC-CRIT-01: Reject plaintext admin token in strict mode (default on)
    api::validate_admin_token().unwrap_or_else(|e| {
        error!("{}\nExiting Vaultwarden!", e.message());
        exit(1);
    });
    // SEC-CRIT-02: Require double-confirmation before allowing unauthenticated admin panel
    api::validate_disable_admin_token().unwrap_or_else(|e| {
        error!("{}\nExiting Vaultwarden!", e.message());
        exit(1);
    });

    check_web_vault();

    create_dir(&CONFIG.tmp_folder(), "tmp folder");

    let pool = create_db_pool().await;
    audit::init_audit_log(pool.clone());
    siem::SiemForwarder::start(pool.clone());
    // TASK-008-011: Initialize webhook delivery global pool
    webhook_delivery::init_pool(pool.clone());

    // LOW-04-A: schedule_jobs is now async (tokio-cron-scheduler); spawn as a task
    tokio::spawn(schedule_jobs(pool.clone()));
    // TASK-RUSTDEV-HIGH-02: Start the WS DashMap cleanup task
    api::start_ws_cleanup_task();
    // TASK-010-015: Start security alerting engine
    tokio::spawn(alerting::start_alerting_engine());
    
    #[cfg(feature = "redis")]
    api::start_redis_pubsub_listener();

    db::models::TwoFactor::migrate_u2f_to_webauthn(&pool.get().await.unwrap()).await.unwrap();
    db::models::TwoFactor::migrate_credential_to_passkey(&pool.get().await.unwrap()).await.unwrap();

    let extra_debug = matches!(level, log::LevelFilter::Trace | log::LevelFilter::Debug);
    launch_rocket(pool, extra_debug).await // Blocks until program termination.
}

const HELP: &str = "\
Alternative implementation of the Bitwarden server API written in Rust

USAGE:
    vaultwarden [FLAGS|COMMAND]

FLAGS:
    -h, --help       Prints help information
    -v, --version    Prints the app and web-vault version

COMMAND:
    hash [--preset {bitwarden|owasp}]  Generate an Argon2id PHC ADMIN_TOKEN
    backup                             Create a backup of the SQLite database
                                       You can also send the USR1 signal to trigger a backup

PRESETS:                  m=         t=          p=
    bitwarden (default) 64MiB, 3 Iterations, 4 Threads
    owasp               19MiB, 2 Iterations, 1 Thread

// TASK-SEC-CRIT-01-D: Clear guidance on using the generated token
GENERATING AN ADMIN TOKEN:
    1. Run:  vaultwarden hash --preset owasp
    2. Copy the output 'ADMIN_TOKEN=...' line into your environment or .env file
    3. The token MUST start with '$argon2' — plaintext tokens are rejected by default
    4. Docs: https://github.com/dani-garcia/vaultwarden/wiki/Enabling-admin-page

";

pub const VERSION: Option<&str> = option_env!("VW_VERSION");

fn parse_args() {
    let mut pargs = pico_args::Arguments::from_env();
    let version = VERSION.unwrap_or("(Version info from Git not present)");

    if pargs.contains(["-h", "--help"]) {
        println!("Vaultwarden {version}");
        print!("{HELP}");
        exit(0);
    } else if pargs.contains(["-v", "--version"]) {
        config::SKIP_CONFIG_VALIDATION.store(true, Ordering::Relaxed);
        let web_vault_version = util::get_web_vault_version();
        println!("Vaultwarden {version}");
        println!("Web-Vault {web_vault_version}");
        exit(0);
    }

    if let Some(command) = pargs.subcommand().unwrap_or_default() {
        if command == "hash" {
            use argon2::{
                password_hash::SaltString, Algorithm::Argon2id, Argon2, ParamsBuilder, PasswordHasher, Version::V0x13,
            };

            let mut argon2_params = ParamsBuilder::new();
            let preset: Option<String> = pargs.opt_value_from_str(["-p", "--preset"]).unwrap_or_default();
            let selected_preset;
            match preset.as_deref() {
                Some("owasp") => {
                    selected_preset = "owasp";
                    argon2_params.m_cost(19456);
                    argon2_params.t_cost(2);
                    argon2_params.p_cost(1);
                }
                _ => {
                    // Bitwarden preset is the default
                    selected_preset = "bitwarden";
                    argon2_params.m_cost(65540);
                    argon2_params.t_cost(3);
                    argon2_params.p_cost(4);
                }
            }

            println!("Generate an Argon2id PHC string using the '{selected_preset}' preset:\n");

            let password = rpassword::prompt_password("Password: ").unwrap();
            if password.len() < 8 {
                println!("\nPassword must contain at least 8 characters");
                exit(1);
            }

            let password_verify = rpassword::prompt_password("Confirm Password: ").unwrap();
            if password != password_verify {
                println!("\nPasswords do not match");
                exit(1);
            }

            let argon2 = Argon2::new(Argon2id, V0x13, argon2_params.build().unwrap());
            let salt = SaltString::encode_b64(&crypto::get_random_bytes::<32>()).unwrap();

            let argon2_timer = tokio::time::Instant::now();
            if let Ok(password_hash) = argon2.hash_password(password.as_bytes(), &salt) {
                // TASK-SEC-CRIT-01-D: Guide users on next steps after generating the token
                println!(
                    "\n\
                    ADMIN_TOKEN='{password_hash}'\n\n\
                    Generation of the Argon2id PHC string took: {:?}\n\n\
                    Next steps:\n\
                    1. Copy the ADMIN_TOKEN line above into your .env file or environment\n\
                    2. The token starts with '$argon2' — this format is required (strict mode is default)\n\
                    3. Restart Vaultwarden to apply\n\
                    4. Docs: https://github.com/dani-garcia/vaultwarden/wiki/Enabling-admin-page",
                    argon2_timer.elapsed()
                );
            } else {
                println!("Unable to generate Argon2id PHC hash.");
                exit(1);
            }
        } else if command == "backup" {
            match db::backup_sqlite() {
                Ok(f) => {
                    println!("Backup to '{f}' was successful");
                    exit(0);
                }
                Err(e) => {
                    println!("Backup failed. {e:?}");
                    exit(1);
                }
            }
        }
        exit(0);
    }
}

fn launch_info() {
    println!(
        "\
        /--------------------------------------------------------------------\\\n\
        |                        Starting Vaultwarden                        |"
    );

    if let Some(version) = VERSION {
        println!("|{:^68}|", format!("Version {version}"));
    }

    println!(
        "\
        |--------------------------------------------------------------------|\n\
        | This is an *unofficial* Bitwarden implementation, DO NOT use the   |\n\
        | official channels to report bugs/features, regardless of client.   |\n\
        | Send usage/configuration questions or feature requests to:         |\n\
        |   https://github.com/dani-garcia/vaultwarden/discussions or        |\n\
        |   https://vaultwarden.discourse.group/                             |\n\
        | Report suspected bugs/issues in the software itself at:            |\n\
        |   https://github.com/dani-garcia/vaultwarden/issues/new            |\n\
        \\--------------------------------------------------------------------/\n"
    );
}

fn init_logging() -> Result<log::LevelFilter, Error> {
    let levels = log::LevelFilter::iter().map(|lvl| lvl.as_str().to_lowercase()).collect::<Vec<String>>().join("|");
    let log_level_rgx_str = format!("^({levels})((,[^,=]+=({levels}))*)$");
    let log_level_rgx = regex::Regex::new(&log_level_rgx_str)?;
    let config_str = CONFIG.log_level().to_lowercase();

    let (level, levels_override) = if let Some(caps) = log_level_rgx.captures(&config_str) {
        let level = caps
            .get(1)
            .and_then(|m| log::LevelFilter::from_str(m.as_str()).ok())
            .ok_or(Error::new("Failed to parse global log level".to_string(), ""))?;

        let levels_override: Vec<(&str, log::LevelFilter)> = caps
            .get(2)
            .map(|m| {
                m.as_str()
                    .split(',')
                    .collect::<Vec<&str>>()
                    .into_iter()
                    .flat_map(|s| match s.split_once('=') {
                        Some((log, lvl_str)) => log::LevelFilter::from_str(lvl_str).ok().map(|lvl| (log, lvl)),
                        _ => None,
                    })
                    .collect()
            })
            .ok_or(Error::new("Failed to parse overrides".to_string(), ""))?;

        (level, levels_override)
    } else {
        err!(format!("LOG_LEVEL should follow the format info,vaultwarden::api::icons=debug, invalid: {config_str}"))
    };

    // Depending on the main log level we either want to disable or enable logging for hickory.
    // Else if there are timeouts it will clutter the logs since hickory uses warn for this.
    let hickory_level = if level >= log::LevelFilter::Debug {
        level
    } else {
        log::LevelFilter::Off
    };

    // Only show Rocket underscore `_` logs when the level is Debug or higher
    // Else this will bloat the log output with useless messages.
    let rocket_underscore_level = if level >= log::LevelFilter::Debug {
        log::LevelFilter::Warn
    } else {
        log::LevelFilter::Off
    };

    // Only show handlebar logs when the level is Trace
    let handlebars_level = if level >= log::LevelFilter::Trace {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Warn
    };

    // Enable smtp debug logging only specifically for smtp when need.
    // This can contain sensitive information we do not want in the default debug/trace logging.
    let smtp_log_level = if CONFIG.smtp_debug() {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Off
    };

    let mut default_levels = HashMap::from([
        // Hide unknown certificate errors if using self-signed
        ("rustls::session", log::LevelFilter::Off),
        // Hide failed to close stream messages
        ("hyper::server", log::LevelFilter::Warn),
        // Silence Rocket `_` logs
        ("_", rocket_underscore_level),
        ("rocket::response::responder::_", rocket_underscore_level),
        ("rocket::server::_", rocket_underscore_level),
        ("vaultwarden::api::admin::_", rocket_underscore_level),
        ("vaultwarden::api::notifications::_", rocket_underscore_level),
        // Silence Rocket logs
        ("rocket::launch", log::LevelFilter::Error),
        ("rocket::launch_", log::LevelFilter::Error),
        ("rocket::rocket", log::LevelFilter::Warn),
        ("rocket::server", log::LevelFilter::Warn),
        ("rocket::fairing::fairings", log::LevelFilter::Warn),
        ("rocket::shield::shield", log::LevelFilter::Warn),
        ("hyper::proto", log::LevelFilter::Off),
        ("hyper::client", log::LevelFilter::Off),
        // Filter handlebars logs
        ("handlebars::render", handlebars_level),
        // Prevent cookie_store logs
        ("cookie_store", log::LevelFilter::Off),
        // Variable level for hickory used by reqwest
        ("hickory_resolver::name_server::name_server", hickory_level),
        ("hickory_proto::xfer", hickory_level),
        // SMTP
        ("lettre::transport::smtp", smtp_log_level),
        // Set query_logger default to Off, but can be overwritten manually
        // You can set LOG_LEVEL=info,vaultwarden::db::query_logger=<LEVEL> to overwrite it.
        // This makes it possible to do the following:
        // warn = Print slow queries only, 5 seconds or longer
        // info = Print slow queries only, 1 second or longer
        // debug = Print all queries
        ("vaultwarden::db::query_logger", log::LevelFilter::Off),
    ]);

    for (path, level) in levels_override.into_iter() {
        let _ = default_levels.insert(path, level);
    }

    if Some(&log::LevelFilter::Debug) == default_levels.get("lettre::transport::smtp") {
        println!(
            "[WARNING] SMTP Debugging is enabled (SMTP_DEBUG=true). Sensitive information could be disclosed via logs!\n\
             [WARNING] Only enable SMTP_DEBUG during troubleshooting!\n"
        );
    }

    let mut logger = fern::Dispatch::new().level(level).chain(std::io::stdout());

    for (path, level) in default_levels {
        logger = logger.level_for(path.to_string(), level);
    }

    if CONFIG.extended_logging() {
        logger = logger.format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format(&CONFIG.log_timestamp_format()),
                record.target(),
                record.level(),
                message
            ))
        });
    } else {
        logger = logger.format(|out, message, _| out.finish(format_args!("{message}")));
    }

    if let Some(log_file) = CONFIG.log_file() {
        #[cfg(windows)]
        {
            logger = logger.chain(fern::log_file(log_file)?);
        }
        #[cfg(unix)]
        {
            const SIGHUP: i32 = SignalKind::hangup().as_raw_value();
            let path = Path::new(&log_file);
            logger = logger.chain(fern::log_reopen1(path, [SIGHUP])?);
        }
    }

    #[cfg(unix)]
    {
        if cfg!(feature = "enable_syslog") || CONFIG.use_syslog() {
            logger = chain_syslog(logger);
        }
    }

    // TASK-010-011: JSON structured logging
    // When LOG_FORMAT=json, bypass fern and install tracing-subscriber with JSON output.
    // The tracing → log bridge (tracing::log feature) ensures existing `log::` calls are
    // captured by the tracing subscriber, preserving full backward compatibility.
    if CONFIG.log_format().eq_ignore_ascii_case("json") {
        use tracing_subscriber::{fmt, EnvFilter};

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level.as_str()));

        let builder = fmt()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_env_filter(env_filter);

        if CONFIG.log_include_trace_id() {
            builder
                .with_thread_ids(true)
                .init();
        } else {
            builder.init();
        }

        // Also install the log→tracing bridge so `log::info!()` macros are captured
        let _ = tracing_log::LogTracer::init();

        return Ok(level);
    }

    if let Err(err) = logger.apply() {
        err!(format!("Failed to activate logger: {err}"))
    }

    // Catch panics and log them instead of default output to StdErr
    panic::set_hook(Box::new(|info| {
        let thread = thread::current();
        let thread = thread.name().unwrap_or("unnamed");

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };

        let backtrace = std::backtrace::Backtrace::force_capture();

        match info.location() {
            Some(location) => {
                error!(
                    target: "panic", "thread '{}' panicked at '{}': {}:{}\n{:}",
                    thread,
                    msg,
                    location.file(),
                    location.line(),
                    backtrace
                );
            }
            None => error!(
                target: "panic",
                "thread '{thread}' panicked at '{msg}'\n{backtrace:}"
            ),
        }
    }));

    Ok(level)
}

#[cfg(unix)]
fn chain_syslog(logger: fern::Dispatch) -> fern::Dispatch {
    let syslog_fmt = syslog::Formatter3164 {
        facility: syslog::Facility::LOG_USER,
        hostname: None,
        process: "vaultwarden".into(),
        pid: 0,
    };

    match syslog::unix(syslog_fmt) {
        Ok(sl) => logger.chain(sl),
        Err(e) => {
            error!("Unable to connect to syslog: {e:?}");
            logger
        }
    }
}

fn create_dir(path: &str, description: &str) {
    // Try to create the specified dir, if it doesn't already exist.
    let err_msg = format!("Error creating {description} directory '{path}'");
    create_dir_all(path).expect(&err_msg);
}

async fn check_data_folder() {
    let data_folder = &CONFIG.data_folder();

    if data_folder.starts_with("s3://") {
        if let Err(e) = CONFIG
            .opendal_operator_for_path_type(&PathType::Data)
            .unwrap_or_else(|e| {
                error!("Failed to create S3 operator for data folder '{data_folder}': {e:?}");
                exit(1);
            })
            .check()
            .await
        {
            error!("Could not access S3 data folder '{data_folder}': {e:?}");
            exit(1);
        }

        return;
    }

    let path = Path::new(data_folder);
    if !path.exists() {
        error!("Data folder '{data_folder}' doesn't exist.");
        if is_running_in_container() {
            error!("Verify that your data volume is mounted at the correct location.");
        } else {
            error!("Create the data folder and try again.");
        }
        exit(1);
    }
    if !path.is_dir() {
        error!("Data folder '{data_folder}' is not a directory.");
        exit(1);
    }

    if is_running_in_container()
        && std::env::var("I_REALLY_WANT_VOLATILE_STORAGE").is_err()
        && !container_data_folder_is_persistent(data_folder).await
    {
        error!(
            "No persistent volume!\n\
            ########################################################################################\n\
            # It looks like you did not configure a persistent volume!                             #\n\
            # This will result in permanent data loss when the container is removed or updated!    #\n\
            # If you really want to use volatile storage set `I_REALLY_WANT_VOLATILE_STORAGE=true` #\n\
            ########################################################################################\n"
        );
        exit(1);
    }
}

/// Detect when using Docker or Podman the DATA_FOLDER is either a bind-mount or a volume created manually.
/// If not created manually, then the data will not be persistent.
/// A none persistent volume in either Docker or Podman is represented by a 64 alphanumerical string.
/// If we detect this string, we will alert about not having a persistent self defined volume.
/// This probably means that someone forgot to add `-v /path/to/vaultwarden_data/:/data`
async fn container_data_folder_is_persistent(data_folder: &str) -> bool {
    if let Ok(mountinfo) = File::open("/proc/self/mountinfo").await {
        // Since there can only be one mountpoint to the DATA_FOLDER
        // We do a basic check for this mountpoint surrounded by a space.
        let data_folder_match = if data_folder.starts_with('/') {
            format!(" {data_folder} ")
        } else {
            format!(" /{data_folder} ")
        };
        let mut lines = BufReader::new(mountinfo).lines();
        let re = regex::Regex::new(r"/volumes/[a-z0-9]{64}/_data /").unwrap();
        while let Some(line) = lines.next_line().await.unwrap_or_default() {
            // Only execute a regex check if we find the base match
            if line.contains(&data_folder_match) {
                if re.is_match(&line) {
                    return false;
                }
                // If we did found a match for the mountpoint, but not the regex, then still stop searching.
                break;
            }
        }
    }
    // In all other cases, just assume a true.
    // This is just an informative check to try and prevent data loss.
    true
}

fn check_web_vault() {
    if !CONFIG.web_vault_enabled() {
        return;
    }

    let index_path = Path::new(&CONFIG.web_vault_folder()).join("index.html");

    if !index_path.exists() {
        error!(
            "Web vault is not found at '{}'. To install it, please follow the steps in: ",
            CONFIG.web_vault_folder()
        );
        error!("https://github.com/dani-garcia/vaultwarden/wiki/Building-binary#install-the-web-vault");
        error!("You can also set the environment variable 'WEB_VAULT_ENABLED=false' to disable it");
        exit(1);
    }
}

async fn create_db_pool() -> db::DbPool {
    match util::retry_db(db::DbPool::from_config, CONFIG.db_connection_retries()).await {
        Ok(p) => p,
        Err(e) => {
            error!("Error creating database pool: {e:?}");
            exit(1);
        }
    }
}

/// TASK-RUSTDEV-LOW-02-C: Build the Rocket instance (pre-ignition).
///
/// Separated from `launch_rocket` so integration tests can call:
/// ```rust
/// let pool = create_test_db_pool().await;
/// let state = AppState { rate_limiter: Arc::new(NoopRateLimiter) };
/// let client = Client::tracked(build_rocket(pool, state, false)).await.unwrap();
/// ```
/// The returned `Rocket<Build>` has all routes mounted and all managed state
/// attached but has NOT been ignited or launched — tests call `.ignite()` via
/// `Client::tracked()`.
pub fn build_rocket(pool: db::DbPool, state: app_state::AppState, extra_debug: bool) -> rocket::Rocket<rocket::Build> {
    let basepath = &CONFIG.domain_path();

    let mut config = rocket::Config::from(rocket::Config::figment());
    config.temp_dir = canonicalize(CONFIG.tmp_folder()).unwrap_or_default().into();
    config.cli_colors = false; // Make sure Rocket does not color any values for logging.
    config.limits = Limits::new()
        .limit("json", 20.megabytes()) // 20MB should be enough for very large imports, something like 5000+ vault entries
        .limit("data-form", 525.megabytes()) // This needs to match the maximum allowed file size for Send
        .limit("file", 525.megabytes()); // This needs to match the maximum allowed file size for attachments

    // If adding more paths here, consider also adding them to
    // crate::utils::LOGGED_ROUTES to make sure they appear in the log
    rocket::custom(config)
        .mount([basepath, "/"].concat(), api::web_routes())
        .mount([basepath, "/"].concat(), api::compliance_routes())
        .mount([basepath, "/api"].concat(), api::core_routes())
        .mount([basepath, "/admin"].concat(), api::admin_routes())
        .mount([basepath, "/events"].concat(), api::core_events_routes())
        .mount([basepath, "/identity"].concat(), api::identity_routes())
        .mount([basepath, "/icons"].concat(), api::icons_routes())
        .mount([basepath, "/notifications"].concat(), api::notifications_routes())
        .mount([basepath, ""].concat(), api::metrics_routes())
        .mount([basepath, "/scim"].concat(), api::scim_routes())
        .mount([basepath, "/health"].concat(), api::health_routes())
        .mount([basepath, "/api/system"].concat(), api::system_routes())
        .register([basepath, "/"].concat(), api::web_catchers())
        .register([basepath, "/api"].concat(), api::core_catchers())
        .register([basepath, "/admin"].concat(), api::admin_catchers())
        .manage(pool)
        .manage(Arc::clone(&WS_USERS))
        .manage(Arc::clone(&WS_ANONYMOUS_SUBSCRIPTIONS))
        .manage(state)
        .attach(util::AppHeaders())
        .attach(util::MetricsFairing)
        .attach(util::Cors())
        .attach(util::BetterLogging(extra_debug))
}

async fn launch_rocket(pool: db::DbPool, extra_debug: bool) -> Result<(), Error> {
    let instance = build_rocket(pool, app_state::AppState::new(), extra_debug)
        .ignite()
        .await?;

    CONFIG.set_rocket_shutdown_handle(instance.shutdown());

    tokio::spawn(async move {
        #[cfg(unix)]
        {
            let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
            tokio::select! {
                _ = tokio::signal::ctrl_c() => info!("Received Ctrl-C"),
                _ = sigterm.recv() => info!("Received SIGTERM"),
            }
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await.expect("Error setting Ctrl-C handler");
            info!("Received Ctrl-C");
        }
        info!("Exiting Vaultwarden! Initiating graceful shutdown...");
        CONFIG.shutdown();
    });

    #[cfg(all(unix, sqlite))]
    {
        if db::ACTIVE_DB_TYPE.get() != Some(&db::DbConnType::Sqlite) {
            debug!("PostgreSQL and MySQL/MariaDB do not support this backup feature, skip adding USR1 signal.");
        } else {
            tokio::spawn(async move {
                let mut signal_user1 = tokio::signal::unix::signal(SignalKind::user_defined1()).unwrap();
                loop {
                    // If we need more signals to act upon, we might want to use select! here.
                    // With only one item to listen for this is enough.
                    let _ = signal_user1.recv().await;
                    match db::backup_sqlite() {
                        Ok(f) => info!("Backup to '{f}' was successful"),
                        Err(e) => error!("Backup failed. {e:?}"),
                    }
                }
            });
        }
    }

    instance.launch().await?;

    info!("Vaultwarden process exited! Rocket graceful shutdown completed.");
    // TASK-005-014: Rocket handles tying off existing open sockets gracefully.
    // The DbPool and Cache adapters are dropped cleanly when the Rocket instance is dropped here.
    Ok(())
}

/// TASK-RUSTDEV-LOW-04-A: Async job scheduler using tokio-cron-scheduler.
///
/// Replaces the synchronous `job_scheduler_ng` with `tokio-cron-scheduler` 0.13,
/// which runs each job as a native tokio task — no `Arc<Runtime>` or `thread::spawn`
/// required.  All jobs still use `catch_unwind` to prevent a panicking job from
/// killing the scheduler loop.
async fn schedule_jobs(pool: db::DbPool) {
    if CONFIG.job_poll_interval_ms() == 0 {
        info!("Job scheduler disabled.");
        return;
    }

    use tokio_cron_scheduler::{Job, JobScheduler};

    let sched = match JobScheduler::new().await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create job scheduler: {e:?}");
            return;
        }
    };

    // Purge sends that are past their deletion date.
    if !CONFIG.send_purge_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.send_purge_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(api::purge_sends(p));
                })) {
                    error!("Job 'purge_sends' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'purge_sends' job: {e:?}"),
        }
    }

    // Purge trashed items that are old enough to be auto-deleted.
    if !CONFIG.trash_purge_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.trash_purge_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(api::purge_trashed_ciphers(p));
                })) {
                    error!("Job 'purge_trashed_ciphers' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'purge_trashed_ciphers' job: {e:?}"),
        }
    }

    // TASK-001-008: GDPR right to erasure scheduled job
    if !CONFIG.gdpr_erasure_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.gdpr_erasure_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(api::execute_scheduled_erasures(p));
                })) {
                    error!("Job 'execute_scheduled_erasures' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'execute_scheduled_erasures' job: {e:?}"),
        }
    }

    // TASK-006-008: Register backup scheduler
    if CONFIG.backup_enabled() && !CONFIG.backup_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.backup_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(async move {
                        let bm = backup::BackupManager::new();
                        if let Ok(mut conn) = p.get().await {
                            if let Err(err) = bm.run_backup(&mut conn).await {
                                error!("Scheduled backup failed: {}", err);
                            }
                        } else {
                            error!("Scheduled backup failed: Could not acquire DB connection");
                        }
                    });
                })) {
                    error!("Job 'backup_scheduler' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'backup_scheduler' job: {e:?}"),
        }
    }

    // TASK-002-017: Audit retention archival job
    if !CONFIG.audit_retention_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.audit_retention_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(audit::archive_older_than_job(p));
                })) {
                    error!("Job 'archive_older_than_job' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'archive_older_than_job' job: {e:?}"),
        }
    }

    // Send email notifications about incomplete 2FA logins.
    if !CONFIG.incomplete_2fa_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.incomplete_2fa_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(api::send_incomplete_2fa_notifications(p));
                })) {
                    error!("Job 'send_incomplete_2fa_notifications' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'send_incomplete_2fa_notifications' job: {e:?}"),
        }
    }

    // Grant emergency access requests that have met the required wait time.
    if !CONFIG.emergency_request_timeout_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.emergency_request_timeout_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(api::emergency_request_timeout_job(p));
                })) {
                    error!("Job 'emergency_request_timeout_job' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'emergency_request_timeout_job' job: {e:?}"),
        }
    }

    // Send reminders to emergency access grantors.
    if !CONFIG.emergency_notification_reminder_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.emergency_notification_reminder_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(api::emergency_notification_reminder_job(p));
                })) {
                    error!("Job 'emergency_notification_reminder_job' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'emergency_notification_reminder_job' job: {e:?}"),
        }
    }

    // Purge old auth requests.
    if !CONFIG.auth_request_purge_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.auth_request_purge_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(purge_auth_requests(p));
                })) {
                    error!("Job 'purge_auth_requests' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'purge_auth_requests' job: {e:?}"),
        }
    }

    // Clean unused, expired Duo authentication contexts.
    if !CONFIG.duo_context_purge_schedule().is_empty() && CONFIG._enable_duo() && !CONFIG.duo_use_iframe() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.duo_context_purge_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(purge_duo_contexts(p));
                })) {
                    error!("Job 'purge_duo_contexts' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'purge_duo_contexts' job: {e:?}"),
        }
    }

    // Cleanup the event table of records older than events_days_retain.
    if CONFIG.org_events_enabled()
        && !CONFIG.event_cleanup_schedule().is_empty()
        && CONFIG.events_days_retain().is_some()
    {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.event_cleanup_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(api::event_cleanup_job(p));
                })) {
                    error!("Job 'event_cleanup_job' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'event_cleanup_job' job: {e:?}"),
        }
    }

    // Purge SSO nonce from incomplete flows.
    if !CONFIG.purge_incomplete_sso_nonce().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.purge_incomplete_sso_nonce().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(db::models::SsoNonce::delete_expired(p));
                })) {
                    error!("Job 'delete_expired_sso_nonce' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'delete_expired_sso_nonce' job: {e:?}"),
        }
    }

    // TASK-SEC-HIGH-02-G: Daily cleanup of expired revoked tokens.
    // Only registered when TOKEN_REVOCATION_ENABLED=true to keep minimal overhead.
    if CONFIG.token_revocation_enabled() {
        let pool = pool.clone();
        let job = Job::new_async("0 0 3 * * *", move |_uuid, _lock| {
            // Run at 03:00 UTC daily
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(async move {
                        if let Ok(conn) = p.get().await {
                            if let Err(e) = db::models::RevokedToken::delete_expired(&conn).await {
                                error!("Failed to delete expired revoked tokens: {e}");
                            } else {
                                debug!("Expired revoked tokens cleaned up successfully");
                            }
                        } else {
                            error!("Failed to get DB connection for revoked token cleanup");
                        }
                    });
                })) {
                    error!("Job 'revoked_token_cleanup' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => {
                let _ = sched.add(j).await;
            }
            Err(e) => error!("Failed to add 'revoked_token_cleanup' job: {e:?}"),
        }
    }

    // TASK-SEC-LOW-01-B: Automatic JWT signing key rotation job.
    // Only registered when JWT_KEY_ROTATION_SCHEDULE is non-empty.
    // WARNING: Each rotation forces ALL users to re-authenticate.
    if !CONFIG.jwt_key_rotation_schedule().is_empty() {
        let pool = pool.clone();
        let job = Job::new_async(CONFIG.jwt_key_rotation_schedule().as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                info!("[KeyRotation] Scheduled JWT key rotation starting...");
                match auth::rotate_jwt_signing_key().await {
                    Ok(_new_pub_key) => {
                        // Invalidate all sessions after key rotation
                        if let Ok(conn) = p.get().await {
                            let all_users = db::models::User::get_all(&conn).await;
                            let count = all_users.len();
                            for (mut user, _) in all_users {
                                user.reset_security_stamp();
                                if let Err(e) = user.save(&conn).await {
                                    error!("[KeyRotation] Error resetting stamp for {}: {e}", user.uuid);
                                }
                            }
                            warn!("[KeyRotation] Scheduled rotation complete. {} sessions invalidated.", count);
                        } else {
                            error!("[KeyRotation] Failed to get DB connection for session invalidation");
                        }
                    }
                    Err(e) => error!("[KeyRotation] Scheduled key rotation failed: {e}"),
                }
            })
        });
        match job {
            Ok(j) => {
                let _ = sched.add(j).await;
                info!("[KeyRotation] Automatic JWT key rotation scheduled: {}", CONFIG.jwt_key_rotation_schedule());
            }
            Err(e) => error!("Failed to add 'jwt_key_rotation' job: {e:?}"),
        }
    }
    // TASK-003: LDAP Sync Job
    // Since this runs repeatedly every N minutes, we use the equivalent cron format
    // or just run tokio::time interval in a separate spawned task. With tokio-cron-scheduler,
    // we can use a repeating interval or a cron expression. A simple cron every N minutes:
    if CONFIG.ldap_enabled() {
        let pool = pool.clone();
        let interval_mins = CONFIG.ldap_sync_interval_minutes();
        let cron_expr = format!("0 */{} * * * *", interval_mins); // e.g., "0 */60 * * * *"
        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(ldap::ldap_sync_job(p));
                })) {
                    error!("Job 'ldap_sync_job' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'ldap_sync_job' job: {e:?}"),
        }
    }
    
    // TASK-003: Access Review Job — creates new reviews periodically
    if CONFIG.access_review_enabled() {
        let pool1 = pool.clone();
        let pool2 = pool.clone();

        let interval_days = CONFIG.access_review_interval_days();
        let cron_expr = format!("0 0 0 */{} * *", interval_days.max(1));
        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let p = pool1.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(db::models::access_review::access_review_job(p));
                })) {
                    error!("Job 'access_review_job' panicked: {:?}", e);
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'access_review_job' job: {e:?}"),
        }

        // TASK-003-019: deadline / auto-revoke — runs daily at 01:00 UTC
        let deadline_job = Job::new_async("0 0 1 * * *", move |_uuid, _lock| {
            let p = pool2.clone();
            Box::pin(async move {
                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    tokio::spawn(db::models::access_review::access_review_deadline_job(p));
                })) {
                    error!("Job 'access_review_deadline_job' panicked: {:?}", e);
                }
            })
        });
        match deadline_job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'access_review_deadline_job': {e:?}"),
        }
    }
    // TASKS-SOL-006: Backup Scheduler Job
    if CONFIG.backup_enabled() {
        let pool = pool.clone();
        let cron_expr = CONFIG.backup_schedule();
        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let _p = pool.clone();
            Box::pin(async move {
                // let manager = crate::backup::BackupManager::new();
                // let _ = manager.run_backup().await;
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'backup_job': {e:?}"),
        }
    }

    // TASKS-SOL-007: PAM Rotation & Auto-Expiry Checks
    if CONFIG.pam_enabled() {
        // Runs every minute
        let job = Job::new_async("0 * * * * *", move |_uuid, _lock| {
            Box::pin(async move {
                // TODO: trigger checkout expiry checks
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'pam_expiry_job': {e:?}"),
        }
    }

    // TASKS-SOL-008: Webhook Delivery Retries
    if CONFIG.webhook_enabled() {
        // Runs every 5 minutes
        let job = Job::new_async("0 */5 * * * *", move |_uuid, _lock| {
            Box::pin(async move {
                // TODO: implement webhook retry delivery engine
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'webhook_retry_job': {e:?}"),
        }
    }

    // TASKS-SOL-009: Device Cert Expiration Warn
    if CONFIG.device_trust_enabled() {
        let pool = pool.clone();
        // Runs daily at 2:00 AM
        let job = Job::new_async("0 0 2 * * *", move |_uuid, _lock| {
            let p = pool.clone();
            Box::pin(async move {
                #[allow(unused_mut)]
                if let Ok(mut conn) = p.get().await {
                    use chrono::{Utc, Duration};
                    let upcoming = Utc::now().naive_utc() + Duration::days(14);
                    use crate::db::schema::devices;
                    use diesel::prelude::*;
                    use crate::db_run;
                    
                    let expiring_devices = db_run! { conn: {
                        devices::table
                            .filter(devices::cert_expires_at.lt(upcoming))
                            .filter(devices::cert_expires_at.gt(Utc::now().naive_utc()))
                            .load::<db::models::Device>(conn)
                            .unwrap_or_default()
                    }};
                    
                    for dev in expiring_devices {
                        warn!("ALERT: Device {} (User {}) cert expires at {:?}", dev.uuid, dev.user_uuid, dev.cert_expires_at);
                    }
                }
            })
        });
        match job {
            Ok(j) => { let _ = sched.add(j).await; }
            Err(e) => error!("Failed to add 'device_cert_warn_job': {e:?}"),
        }
    }


    if let Err(e) = sched.start().await {
        error!("Failed to start job scheduler: {e:?}");
    }
}
