use rocket::serde::json::{Json, Value};
use rocket::serde::json::serde_json::json;
use rocket::State;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, NotSet, PaginatorTrait, QueryFilter, QueryTrait};
use sea_orm::ActiveValue::Set;

use crate::model::{sys_role, sys_role_menu, sys_user_role};
use crate::model::prelude::{SysMenu, SysRole, SysRoleMenu, SysUserRole};
use crate::utils::auth::Token;
use crate::vo::{err_result_msg, err_result_page, handle_result, ok_result_data, ok_result_page};
use crate::vo::role_vo::*;

// 查询角色列表
#[post("/role_list", data = "<item>")]
pub async fn role_list(db: &State<DatabaseConnection>, item: Json<RoleListReq>, _auth: Token) -> Value {
    log::info!("role_list params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let paginator = SysRole::find()
        .apply_if(item.role_name.clone(), |mut query, v| {
            query.filter(sys_role::Column::RoleName.eq(v))
        })
        .apply_if(item.status_id.clone(), |mut query, v| {
            query.filter(sys_role::Column::StatusId.eq(v))
        }).paginate(db, item.page_size.clone());

    let total = paginator.num_items().await.unwrap_or_default();
    let result = paginator.fetch_page(item.page_no.clone() - 1).await;

    match result {
        Ok(page) => {
            let mut role_list: Vec<RoleListData> = Vec::new();

            for role in page {
                role_list.push(RoleListData {
                    id: role.id,
                    sort: role.sort,
                    status_id: role.status_id,
                    role_name: role.role_name,
                    remark: role.remark,
                    create_time: role.create_time.to_string(),
                    update_time: role.update_time.to_string(),
                })
            }

            json!(ok_result_page(role_list, total))
        }
        Err(err) => {
            json!(err_result_page(err.to_string()))
        }
    }
}

// 添加角色信息
#[post("/role_save", data = "<item>")]
pub async fn role_save(db: &State<DatabaseConnection>, item: Json<RoleSaveReq>, _auth: Token) -> Value {
    log::info!("role_save params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let role = item.0;

    let sys_role = sys_role::ActiveModel {
        id: NotSet,
        status_id: Set(role.status_id),
        sort: Set(role.sort),
        role_name: Set(role.role_name),
        remark: Set(role.remark.unwrap_or_default()),
        ..Default::default()
    };

    json!(&handle_result(SysRole::insert(sys_role).exec(db).await))
}

// 更新角色信息
#[post("/role_update", data = "<item>")]
pub async fn role_update(db: &State<DatabaseConnection>, item: Json<RoleUpdateReq>, _auth: Token) -> Value {
    log::info!("role_update params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let role = item.0;

    let sys_role = sys_role::ActiveModel {
        id: Set(role.id),
        status_id: Set(role.status_id),
        sort: Set(role.sort),
        role_name: Set(role.role_name),
        remark: Set(role.remark.unwrap_or_default()),
        ..Default::default()
    };

    json!(&handle_result(SysRole::update(sys_role).exec(db).await))
}

// 删除角色信息
#[post("/role_delete", data = "<item>")]
pub async fn role_delete(db: &State<DatabaseConnection>, item: Json<RoleDeleteReq>, _auth: Token) -> Value {
    log::info!("role_delete params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let ids = item.ids.clone();

    if SysUserRole::find().filter(sys_user_role::Column::RoleId.is_in(ids)).count(db).await.unwrap_or_default() > 0 {
        return json!(err_result_msg("角色已被使用,不能直接删除".to_string()));
    }

    json!(&handle_result(SysRole::delete_many().filter(sys_role::Column::Id.is_in(item.ids.clone())).exec(db).await))
}

// 查询角色关联的菜单
#[post("/query_role_menu", data = "<item>")]
pub async fn query_role_menu(db: &State<DatabaseConnection>, item: Json<QueryRoleMenuReq>, _auth: Token) -> Value {
    log::info!("query_role_menu params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let mut menu_data_list: Vec<MenuDataList> = Vec::new();
    let mut role_menu_ids: Vec<i64> = Vec::new();

    for x in SysMenu::find().all(db).await.unwrap_or_default() {
        menu_data_list.push(MenuDataList {
            id: x.id.clone(),
            parent_id: x.parent_id,
            title: x.menu_name.clone(),
            key: x.id.to_string(),
            label: x.menu_name,
            is_penultimate: x.parent_id == 2,
        });
        role_menu_ids.push(x.id)
    }

    //不是超级管理员的时候,就要查询角色和菜单的关联
    if item.role_id != 1 {
        role_menu_ids.clear();

        match SysRoleMenu::find().filter(sys_role_menu::Column::RoleId.eq(item.role_id.clone())).all(db).await {
            Ok(qr) => {
                for x in qr {
                    role_menu_ids.push(x.menu_id);
                }
            }
            Err(err) => {
                error!("err: {:?}",err.to_string());
            }
        }
    }

    json!(ok_result_data(QueryRoleMenuData {
            role_menus: role_menu_ids,
            menu_list: menu_data_list,
        }))
}

// 更新角色关联的菜单
#[post("/update_role_menu", data = "<item>")]
pub async fn update_role_menu(db: &State<DatabaseConnection>, item: Json<UpdateRoleMenuReq>, _auth: Token) -> Value {
    log::info!("update_role_menu params: {:?}", &item);
    let db = db as &DatabaseConnection;
    let role_id = item.role_id.clone();

    match SysRoleMenu::delete_many().filter(sys_role_menu::Column::RoleId.eq(role_id)).exec(db).await {
        Ok(_) => {
            let mut menu_role: Vec<sys_role_menu::ActiveModel> = Vec::new();

            for id in &item.menu_ids {
                let menu_id = id.clone();
                menu_role.push(sys_role_menu::ActiveModel {
                    id: NotSet,
                    status_id: Set(1),
                    sort: Set(1),
                    menu_id: Set(menu_id),
                    role_id: Set(role_id.clone()),
                    ..Default::default()
                })
            }

            json!(&handle_result(SysRoleMenu::insert_many(menu_role).exec(db).await))
        }
        Err(err) => {
            error!("err:{}",err.to_string());
            json!(err_result_msg(err.to_string()))
        }
    }
}
