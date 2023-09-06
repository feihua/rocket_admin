#[macro_use]
extern crate rocket;

use std::net::Ipv4Addr;

use rocket::{Config, Request};
use rocket::serde::json::serde_json::json;
use rocket::serde::json::Value;
use tracing_subscriber::filter;

use crate::handler::{menu_handler, role_handler, user_handler};
use crate::setup::set_up_db;
use crate::utils::auth::Token;

pub mod handler;
pub mod model;
pub mod vo;
pub mod utils;
pub mod setup;

#[get("/ping")]
fn ping(_auth: Token) -> &'static str {
    "pong"
}

#[catch(404)]
fn not_found(req: &Request) -> Value {
    json!({"code": 1,"msg": format!("Sorry, '{}' is not a valid path", req.uri())})
}

#[catch(403)]
fn not_permissions(req: &Request) -> Value {
    json!({"code": 1,"msg": format!("you has no permissions request path: '{}'", req.uri())})
}


#[catch(401)]
fn resp() -> Value {
    json!({"code": 401,"msg": "Unauthorized","description": "The request requires user authentication"})
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    tracing_subscriber::fmt()
        .with_max_level(filter::LevelFilter::DEBUG)
        .with_test_writer()
        .init();
    let db = match set_up_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };

    let config = Config {
        address: Ipv4Addr::new(0, 0, 0, 0).into(),
        port: 8099,
        ..Config::debug_default()
    };

    let _rocket = rocket::build()
        .manage(db)
        .configure(config)
        .mount("/", routes![ping])
        .mount("/api", routes![user_handler::login,
            user_handler::query_user_role,
            user_handler::update_user_role,
            user_handler::query_user_menu,
            user_handler::user_list,
            user_handler::user_save,
            user_handler::user_delete,
            user_handler::user_update,
            user_handler::update_user_password,
            role_handler::query_role_menu,
            role_handler::update_role_menu,
            role_handler::role_list,
            role_handler::role_save,
            role_handler::role_delete,
            role_handler::role_update,
            menu_handler::menu_list,
            menu_handler::menu_save,
            menu_handler::menu_delete,
            menu_handler::menu_update,])
        .register("/", catchers![not_found,resp,not_permissions])
        .launch()
        .await?;

    Ok(())
}