use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::associations::HasTable;
use rocket::serde::json::{Json, Value};
use rocket::serde::json::serde_json::json;

use crate::{RB, schema};
use crate::model::menu::SysMenu;
use crate::model::role::{SysRole, SysRoleAdd, SysRoleUpdate};
use crate::model::role_menu::SysRoleMenuAdd;
use crate::schema::sys_menu::dsl::sys_menu;
use crate::schema::sys_role::{id, role_name, status_id};
use crate::schema::sys_role::dsl::sys_role;
use crate::schema::sys_role_menu::{menu_id, role_id};
use crate::schema::sys_role_menu::dsl::sys_role_menu;
use crate::schema::sys_user_role::dsl::sys_user_role;
use crate::middleware::auth::Token;
use crate::vo::{err_result_msg, handle_result, ok_result_data, ok_result_page};
use crate::vo::role_vo::*;

// 查询角色列表
#[post("/role_list", data = "<item>")]
pub async fn role_list(item: Json<RoleListReq>, _auth: Token) -> Value {
    log::info!("role_list params: {:?}", &item);
    let mut query = sys_role::table().into_boxed();
    if let Some(i) = &item.role_name {
        query = query.filter(role_name.eq(i));
    }
    if let Some(i) = &item.status_id {
        query = query.filter(status_id.eq(i));
    }

    debug!("SQL:{}", diesel::debug_query::<diesel::mysql::Mysql, _>(&query).to_string());

    match &mut RB.clone().get() {
        Ok(conn) => {
            let result = query.load::<SysRole>(conn);
            let mut list: Vec<RoleListData> = Vec::new();
            if let Ok(role_list) = result {
                for role in role_list {
                    list.push(RoleListData {
                        id: role.id,
                        sort: role.sort,
                        status_id: role.status_id,
                        role_name: role.role_name,
                        remark: role.remark,
                        create_time: role.create_time.to_string(),
                        update_time: role.update_time.to_string(),
                    })
                }
            }

            json!(ok_result_page(list, 10))
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            json!(err_result_msg(err.to_string()))
        }
    }
}

// 添加角色信息
#[post("/role_save", data = "<item>")]
pub async fn role_save(item: Json<RoleSaveReq>, _auth: Token) -> Value {
    log::info!("role_save params: {:?}", &item);

    let role = item.0;
    let role_add = SysRoleAdd {
        status_id: role.status_id,
        sort: role.sort,
        role_name: role.role_name,
        remark: role.remark.unwrap(),
    };

    match &mut RB.clone().get() {
        Ok(conn) => {
            json!(handle_result(diesel::insert_into(sys_role::table()).values(role_add).execute(conn)))
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            json!(err_result_msg(err.to_string()))
        }
    }
}

// 更新角色信息
#[post("/role_update", data = "<item>")]
pub async fn role_update(item: Json<RoleUpdateReq>, _auth: Token) -> Value {
    log::info!("role_update params: {:?}", &item);

    let role = item.0;
    let s_role = SysRoleUpdate {
        id: role.id,
        status_id: role.status_id,
        sort: role.sort,
        role_name: role.role_name,
        remark: role.remark.unwrap_or_default(),
    };

    match &mut RB.clone().get() {
        Ok(conn) => {
            json!(handle_result(diesel::update(sys_role).filter(id.eq(&role.id)).set(s_role).execute(conn)))
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            json!(err_result_msg(err.to_string()))
        }
    }
}

// 删除角色信息
#[post("/role_delete", data = "<item>")]
pub async fn role_delete(item: Json<RoleDeleteReq>, _auth: Token) -> Value {
    log::info!("role_delete params: {:?}", &item);
    match &mut RB.clone().get() {
        Ok(conn) => {
            let ids = item.ids.clone();
            //查询角色有没有被使用了,如果使用了就不能删除
            match sys_user_role.filter(schema::sys_user_role::role_id.eq_any(ids)).count().get_result::<i64>(conn) {
                Ok(count) => {
                    if count != 0 {
                        error!("err:{}", "角色已被使用,不能删除".to_string());
                        return json!(err_result_msg("角色已被使用,不能删除".to_string()));
                    }
                    json!(handle_result(diesel::delete(sys_role.filter(id.eq_any(&item.ids))).execute(conn)))
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

// 查询角色关联的菜单
#[post("/query_role_menu", data = "<item>")]
pub async fn query_role_menu(item: Json<QueryRoleMenuReq>, _auth: Token) -> Value {
    log::info!("query_role_menu params: {:?}", &item);
    match &mut RB.clone().get() {
        Ok(conn) => {
            let mut menu_data_list: Vec<MenuDataList> = Vec::new();
            let mut role_menu_ids: Vec<i64> = Vec::new();
            // 查询所有菜单
            match sys_menu.load::<SysMenu>(conn) {
                Ok(menu_list) => {
                    for menu in menu_list {
                        menu_data_list.push(MenuDataList {
                            id: menu.id.clone(),
                            parent_id: menu.parent_id,
                            title: menu.menu_name.clone(),
                            key: menu.id.to_string(),
                            label: menu.menu_name,
                            is_penultimate: menu.parent_id == 2,
                        });
                        role_menu_ids.push(menu.id)
                    }
                }
                Err(err) => {
                    error!("err:{}", err.to_string());
                    return json!(err_result_msg(err.to_string()));
                }
            }

            //不是超级管理员的时候,就要查询角色和菜单的关联
            if item.role_id != 1 {
                role_menu_ids.clear();

                match sys_role_menu.filter(role_id.eq(item.role_id.clone())).select(menu_id).load::<i64>(conn) {
                    Ok(menu_ids) => {
                        role_menu_ids = menu_ids
                    }
                    Err(err) => {
                        error!("err:{}", err.to_string());
                        return json!(err_result_msg(err.to_string()));
                    }
                }
            }

            json!(ok_result_data(QueryRoleMenuData {
                role_menus: role_menu_ids,
                menu_list: menu_data_list,
            }))
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            json!(err_result_msg(err.to_string()))
        }
    }
}

// 更新角色关联的菜单
#[post("/update_role_menu", data = "<item>")]
pub async fn update_role_menu(item: Json<UpdateRoleMenuReq>, _auth: Token) -> Value {
    log::info!("update_role_menu params: {:?}", &item);
    let r_id = item.role_id.clone();
    let menu_ids = item.menu_ids.clone();

    match &mut RB.clone().get() {
        Ok(conn) => {
            match diesel::delete(sys_role_menu.filter(role_id.eq(r_id))).execute(conn) {
                Ok(_) => {
                    let mut role_menu: Vec<SysRoleMenuAdd> = Vec::new();

                    for m_id in menu_ids {
                        role_menu.push(SysRoleMenuAdd {
                            status_id: 1,
                            sort: 1,
                            menu_id: m_id.clone(),
                            role_id: r_id.clone(),
                        })
                    }

                    json!(handle_result(diesel::insert_into(sys_role_menu::table()).values(&role_menu).execute(conn)))
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
