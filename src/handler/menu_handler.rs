use rocket::serde::json::{Json, Value};
use rocket::serde::json::serde_json::json;
use rocket::State;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, NotSet, PaginatorTrait, QueryFilter, QueryOrder};
use sea_orm::ActiveValue::Set;

use crate::model::prelude::SysMenu;
use crate::model::sys_menu;
use crate::model::sys_menu::ActiveModel;
use crate::utils::auth::Token;
use crate::vo::{err_result_msg, ok_result_msg, ok_result_page};
use crate::vo::error_handler::ErrorResponder;
use crate::vo::menu_vo::{*};

// 查询菜单
#[post("/menu_list", data = "<item>")]
pub async fn menu_list(db: &State<DatabaseConnection>, item: Json<MenuListReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("menu_list params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let mut menu_list: Vec<MenuListData> = Vec::new();

    for menu in SysMenu::find().order_by_asc(sys_menu::Column::Sort).all(db).await? {
        menu_list.push(MenuListData {
            id: menu.id,
            sort: menu.sort,
            status_id: menu.status_id,
            parent_id: menu.parent_id,
            menu_name: menu.menu_name.clone(),
            label: menu.menu_name,
            menu_url: menu.menu_url,
            icon: menu.menu_icon.unwrap_or_default(),
            api_url: menu.api_url,
            remark: menu.remark.unwrap_or_default(),
            menu_type: menu.menu_type,
            create_time: menu.create_time.to_string(),
            update_time: menu.update_time.to_string(),
        })
    }

    Ok(json!(ok_result_page(menu_list, 0)))
}

// 添加菜单
#[post("/menu_save", data = "<item>")]
pub async fn menu_save(db: &State<DatabaseConnection>, item: Json<MenuSaveReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("menu_save params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let menu = item.0;

    let sys_menu = ActiveModel {
        id: NotSet,
        status_id: Set(menu.status_id),
        sort: Set(menu.sort),
        parent_id: Set(menu.parent_id.unwrap_or_default()),
        menu_name: Set(menu.menu_name),
        menu_url: Set(menu.menu_url.unwrap_or_default()),
        api_url: Set(menu.api_url.unwrap_or_default()),
        menu_icon: Set(menu.icon),
        remark: Set(menu.remark),
        menu_type: Set(menu.menu_type),
        ..Default::default()
    };

    SysMenu::insert(sys_menu).exec(db).await?;
    Ok(json!(ok_result_msg("添加菜单信息成功!")))
}

// 更新菜单
#[post("/menu_update", data = "<item>")]
pub async fn menu_update(db: &State<DatabaseConnection>, item: Json<MenuUpdateReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("menu_update params: {:?}", &item);
    let db = db as &DatabaseConnection;
    let menu = item.0;

    if SysMenu::find_by_id(menu.id.clone()).one(db).await?.is_none() {
        return Ok(json!(err_result_msg("菜单不存在,不能更新!")));
    }

    let sys_menu = ActiveModel {
        id: Set(menu.id),
        status_id: Set(menu.status_id),
        sort: Set(menu.sort),
        parent_id: Set(menu.parent_id.unwrap_or_default()),
        menu_name: Set(menu.menu_name),
        menu_url: Set(menu.menu_url.unwrap_or_default()),
        api_url: Set(menu.api_url.unwrap_or_default()),
        menu_icon: Set(menu.icon),
        remark: Set(menu.remark),
        menu_type: Set(menu.menu_type),
        ..Default::default()
    };

    SysMenu::update(sys_menu).exec(db).await?;
    Ok(json!(ok_result_msg("更新菜单信息成功!")))
}

// 删除菜单信息
#[post("/menu_delete", data = "<item>")]
pub async fn menu_delete(db: &State<DatabaseConnection>, item: Json<MenuDeleteReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("menu_delete params: {:?}", &item);
    let db = db as &DatabaseConnection;

    if SysMenu::find_by_id(item.id.clone()).one(db).await?.is_none() {
        return Ok(json!(err_result_msg("菜单不存在,不能删除!")));
    }

    if SysMenu::find().filter(sys_menu::Column::ParentId.eq(item.id.clone())).count(db).await? > 0 {
        return Ok(json!(err_result_msg("有下级菜单,不能直接删除!")));
    }

    SysMenu::delete_by_id(item.id.clone()).exec(db).await?;
    Ok(json!(ok_result_msg("删除菜单信息成功!")))
}