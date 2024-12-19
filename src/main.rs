#[macro_use]
extern crate rocket;

use std::net::Ipv4Addr;

use crate::handler::system::{sys_menu_handler, sys_role_handler, sys_user_handler};
use crate::setup::set_up_db;
use middleware::auth::Token;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::Value;
use rocket::{Config, Request};
use tracing_subscriber::filter;

pub mod common;
pub mod handler;
pub mod middleware;
pub mod model;
pub mod setup;
pub mod utils;
pub mod vo;

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
        .mount(
            "/api",
            routes![
                sys_user_handler::add_sys_user,
                sys_user_handler::delete_sys_user,
                sys_user_handler::update_sys_user,
                sys_user_handler::update_sys_user_status,
                sys_user_handler::update_user_password,
                sys_user_handler::query_sys_user_detail,
                sys_user_handler::query_sys_user_list,
                sys_user_handler::login,
                sys_user_handler::query_user_role,
                sys_user_handler::update_user_role,
                sys_user_handler::query_user_menu,
                sys_role_handler::add_sys_role,
                sys_role_handler::delete_sys_role,
                sys_role_handler::update_sys_role,
                sys_role_handler::update_sys_role_status,
                sys_role_handler::query_sys_role_detail,
                sys_role_handler::query_sys_role_list,
                sys_role_handler::query_role_menu,
                sys_role_handler::update_role_menu,
                sys_menu_handler::add_sys_menu,
                sys_menu_handler::delete_sys_menu,
                sys_menu_handler::update_sys_menu,
                sys_menu_handler::update_sys_menu_status,
                sys_menu_handler::query_sys_menu_detail,
                sys_menu_handler::query_sys_menu_list,
            ],
        )
        .register("/", catchers![not_found, resp, not_permissions])
        .launch()
        .await?;

    Ok(())
}
