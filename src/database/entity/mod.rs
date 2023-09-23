use mordor_macros::export_seaorm;

pub mod generated;

export_seaorm!(generated::login_entry, "LoginEntry");
export_seaorm!(generated::basic_login_user, "BasicLoginUser");
export_seaorm!(generated::access_entry, "AccessEntry");
