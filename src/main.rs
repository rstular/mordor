use std::sync::Arc;

use actix_session::SessionMiddleware;
use actix_web::{
    cookie::Key,
    middleware::{Logger, NormalizePath, TrailingSlash},
    web, App, HttpServer,
};
use clap::Parser;
use color_eyre::Result;
use tracing::{debug, info, trace};

use crate::{
    controllers::modules::{basic_auth::BasicAuthLoginModule, saml_auth::SAMLLoginModule},
    settings::Settings,
};

mod controllers;
mod database;
mod errors;
mod logging;
mod session;
mod settings;
mod utils;

pub const COOKIE_NAME: &str = "mordor-session";

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[clap(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init()?;
    info!("Initializing mordor...");

    let args = Args::parse();

    let configuration = Arc::new(Settings::load(&args.config)?);
    configuration.sanity_check()?;
    debug!("Loaded configuration from {}", args.config);
    trace!("Loaded configuration: {:#?}", configuration);

    let db_conn = database::init(&configuration.database).await?;
    info!("Database connection established");

    HttpServer::new(enclose! { (db_conn, configuration) move || {
        let login_modules = {
            let mut builder = controllers::ModuleBuilder::new();
            builder.register_module(Box::<BasicAuthLoginModule>::default());
            builder.register_module(Box::new(SAMLLoginModule::new(&configuration.modules.saml)));
            builder.build()
        };

        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Always))
            .wrap(Logger::default())
            .wrap(
                SessionMiddleware::builder(
                    session::CookieTTLSessionStore,
                    Key::from(&configuration.secret_key),
                )
                .cookie_name(COOKIE_NAME.to_string())
                .build(),
            )
            .app_data(web::Data::new(db_conn.clone()))
            .app_data(web::Data::new(configuration.clone()))
            .configure(|sc| controllers::initialize(sc, login_modules))
    }})
    .bind(configuration.http.address)?
    .run()
    .await?;

    Ok(())
}
