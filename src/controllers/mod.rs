use std::sync::Arc;

use actix_session::Session;
use actix_web::{
    get,
    web::{self, ServiceConfig},
    Responder,
};
use lazy_static::lazy_static;
use sea_orm::{DbConn, EntityTrait, Set};
use serde::Serialize;
use tera::Tera;
use tracing::{debug, error};

use crate::{
    database::entity::{AccessEntryActiveModel, AccessEntryEntity},
    errors::{AppError, SessionError},
    settings::Settings,
};

mod login;
pub mod modules;
mod structures;

pub const USERNAME_SESSION_KEY: &str = "username";

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        match tera.add_raw_templates(vec![(
            "login.html",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/_static/login.html"
            )),
        )]) {
            Ok(_) => tera,
            Err(e) => {
                error!("Error parsing templates: {e:?}");
                panic!("Error parsing templates: {e:?}");
            }
        }
    };
}

pub struct ModuleBuilder {
    modules: Vec<Box<dyn LoginModule>>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self { modules: vec![] }
    }

    pub fn register_module(&mut self, module: Box<dyn LoginModule>) {
        self.modules.push(module);
    }

    pub fn build(self) -> Vec<Box<dyn LoginModule>> {
        self.modules
    }
}

pub type AllModuleData = Vec<ModuleData>;
#[derive(Debug, Serialize)]
pub struct ModuleData {
    pub name: &'static str,
    pub subpath: &'static str,
    pub display_name: &'static str,
}

pub trait LoginModule {
    fn name(&self) -> &'static str;
    fn subpath(&self) -> &'static str;
    fn display_name(&self) -> &'static str {
        self.name()
    }
    fn register_controller(&self, svc_cfg: &mut ServiceConfig);
}

pub fn initialize(
    svc_cfg: &mut ServiceConfig,
    modules: impl IntoIterator<Item = Box<dyn LoginModule>>,
) {
    // Initialize login modules
    let mut login_scope = web::scope("/login").service(login::index);
    let mut data: AllModuleData = vec![];
    for module in modules {
        debug!("Registering login module: {}", module.name());

        let subpath = module.subpath();
        assert!(
            subpath.starts_with('/') && !subpath.ends_with('/'),
            "Subpath must start with '/' and must not end with '/'"
        );

        let module_scope =
            web::scope(module.subpath()).configure(|sc| module.register_controller(sc));

        login_scope = login_scope.service(module_scope);

        data.push(ModuleData {
            name: module.name(),
            subpath: &module.subpath()[1..], // Remove leading slash, as it's a relative path, not an absolute one!
            display_name: module.display_name(),
        });
    }
    login_scope = login_scope
        .app_data(web::Data::new(data))
        .service(login::index);

    // Initialize auth scope
    let auth_scope = web::scope("/auth").service(auth);

    svc_cfg.service(login_scope).service(auth_scope);
}

#[get("/")]
async fn auth(
    session: Session,
    db_conn: web::Data<DbConn>,
    configuration: web::Data<Arc<Settings>>,
) -> Result<impl Responder, AppError> {
    let user_id: String = session
        .get(USERNAME_SESSION_KEY)
        .map_err(Into::<SessionError>::into)?
        .ok_or(AppError::NotAuthenticated)?;

    if configuration.store_access_entries {
        let access_entry = AccessEntryActiveModel {
            username: Set(user_id.clone()),
            ..Default::default()
        };
        if let Err(e) = AccessEntryEntity::insert(access_entry)
            .exec(db_conn.as_ref())
            .await
        {
            error!(
                "Error inserting access entry (user_id='{user_id}'): {e}",
                user_id = user_id,
                e = e
            );
        }
    };

    Ok(format!("Logged in as '{user_id}'"))
}
