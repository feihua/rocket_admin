use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::associations::HasTable;
use rocket::serde::json::{Json, Value};
use rocket::serde::json::serde_json::json;

use crate::model::menu::{SysMenu, SysMenuAdd, SysMenuUpdate};
use crate::RB;
use crate::schema::sys_menu::{id, parent_id, sort, status_id};
use crate::schema::sys_menu::dsl::sys_menu;
use crate::middleware::auth::Token;
use crate::vo::{err_result_msg, handle_result, ok_result_page};
use crate::vo::menu_vo::{*};

// 查询菜单
#[post("/menu_list", data = "<item>")]
pub async fn menu_list(item: Json<MenuListReq>, _auth: Token) -> Value {
    log::info!("menu_list params: {:?}", &item);
    match &mut RB.clone().get() {
        Ok(conn) => {
            let mut query = sys_menu::table().into_boxed();
            if let Some(i) = &item.status_id {
                query = query.filter(status_id.eq(i));
            }
            query = query.order(sort.asc());
            debug!("SQL:{}", diesel::debug_query::<diesel::mysql::Mysql, _>(&query).to_string());
            let mut menu_list: Vec<MenuListData> = Vec::new();
            if let Ok(menus) = query.load::<SysMenu>(conn) {
                for menu in menus {
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
            }
            json!(ok_result_page(menu_list, 0))
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            json!(err_result_msg(err.to_string()))
        }
    }
}

// 添加菜单
#[post("/menu_save", data = "<item>")]
pub async fn menu_save(item: Json<MenuSaveReq>, _auth: Token) -> Value {
    log::info!("menu_save params: {:?}", &item);

    let menu = item.0;

    let menu_add = SysMenuAdd {
        status_id: menu.status_id,
        sort: menu.sort,
        parent_id: menu.parent_id.unwrap_or(0),
        menu_name: menu.menu_name,
        menu_url: menu.menu_url,
        api_url: menu.api_url,
        menu_icon: menu.icon,
        remark: menu.remark,
        menu_type: menu.menu_type,
    };

    match &mut RB.clone().get() {
        Ok(conn) => {
            json!(handle_result(diesel::insert_into(sys_menu::table()).values(menu_add).execute(conn)))
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            json!(err_result_msg(err.to_string()))
        }
    }
}

// 更新菜单
#[post("/menu_update", data = "<item>")]
pub async fn menu_update(item: Json<MenuUpdateReq>, _auth: Token) -> Value {
    log::info!("menu_update params: {:?}", &item);
    let menu = item.0;

    let s_menu = SysMenuUpdate {
        id: menu.id,
        status_id: menu.status_id,
        sort: menu.sort,
        parent_id: menu.parent_id,
        menu_name: menu.menu_name,
        menu_url: menu.menu_url,
        api_url: menu.api_url,
        menu_icon: menu.icon,
        remark: menu.remark,
        menu_type: menu.menu_type,
    };

    match &mut RB.clone().get() {
        Ok(conn) => {
            json!(handle_result(diesel::update(sys_menu).filter(id.eq(&menu.id)).set(s_menu).execute(conn)))
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            json!(err_result_msg(err.to_string()))
        }
    }
}

// 删除菜单信息
#[post("/menu_delete", data = "<item>")]
pub async fn menu_delete(item: Json<MenuDeleteReq>, _auth: Token) -> Value {
    log::info!("menu_delete params: {:?}", &item);

    match &mut RB.clone().get() {
        Ok(conn) => {
            match sys_menu.filter(parent_id.eq(item.id.clone())).count().get_result::<i64>(conn) {
                Ok(count) => {
                    if count > 0 {
                        error!("err:{}", "有下级菜单,不能直接删除".to_string());
                        return json!(err_result_msg("有下级菜单,不能直接删除".to_string()));
                    }
                    json!(handle_result(diesel::delete(sys_menu.filter(id.eq(item.id.clone()))).execute(conn)))
                }
                Err(err) => {
                    error!("err:{}", err.to_string());
                    json!(err_result_msg(err.to_string()))
                }
            }
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            json!(err_result_msg(err.to_string()))
        }
    }
}