use rocket::serde::json::{Json, Value};
use rocket::serde::json::serde_json::json;
use rocket::State;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, EntityTrait, NotSet, PaginatorTrait, QueryFilter, QueryOrder, QueryTrait, Statement};
use sea_orm::ActiveValue::Set;

use crate::model::{sys_menu, sys_user, sys_user_role};
use crate::model::prelude::{SysMenu, SysRole, SysUser, SysUserRole};
use crate::utils::auth::Token;
use crate::utils::error::WhoUnfollowedError;
use crate::utils::jwt_util::JWTToken;
use crate::vo::{err_result_msg, err_result_page, handle_result, ok_result_data, ok_result_msg, ok_result_page};
use crate::vo::user_vo::*;

// 后台用户登录
#[post("/login", data = "<item>")]
pub async fn login(db: &State<DatabaseConnection>, item: Json<UserLoginReq>) -> Value {
    log::info!("user login params: {:?}", &item);

    let db = db as &DatabaseConnection;

    let user_result = SysUser::find().filter(sys_user::Column::Mobile.eq(&item.mobile)).one(db).await;
    log::info!("select_by_mobile: {:?}",user_result);

    match user_result {
        Ok(u) => {
            if let Some(user) = u {
                let id = user.id;
                let username = user.user_name;
                let password = user.password;

                if password.ne(&item.password) {
                    return json!(err_result_msg("密码不正确".to_string()));
                }

                let btn_menu = query_btn_menu(db, id).await;

                if btn_menu.len() == 0 {
                    return json!(err_result_msg("用户没有分配角色或者菜单,不能登录".to_string()));
                }

                match JWTToken::new(id, &username, btn_menu).create_token("123") {
                    Ok(token) => {
                        json!(ok_result_data(token))
                    }
                    Err(err) => {
                        let er = match err {
                            WhoUnfollowedError::JwtTokenError(s) => { s }
                            _ => "no math error".to_string()
                        };

                        json!(err_result_msg(er))
                    }
                }
            } else {
                return json!(err_result_msg("用户不存在".to_string()));
            }
        }

        Err(err) => {
            log::info!("select_by_column: {:?}",err);
            return json!(err_result_msg("查询用户异常".to_string()));
        }
    }
}

// 登录的时候 查询权限
async fn query_btn_menu(db: &DatabaseConnection, id: i64) -> Vec<String> {
    let mut btn_menu: Vec<String> = Vec::new();
    //角色Id为1的是系统预留超级管理员角色
    match SysUserRole::find().filter(sys_user_role::Column::UserId.eq(id.clone())).filter(sys_user_role::Column::RoleId.eq(1)).count(db).await {
        Ok(count) => {
            if count != 0 {
                match SysMenu::find().all(db).await {
                    Ok(menu_list) => {
                        for x in menu_list {
                            btn_menu.push(x.api_url);
                        }
                        log::info!("admin login: {:?}",id);
                        btn_menu
                    }
                    Err(err) => {
                        error!("err: {:?}",err.to_string());
                        btn_menu
                    }
                }
            } else {
                let sql_str = r#"select distinct u.api_url from sys_user_role t left join sys_role usr on t.role_id = usr.id left join sys_role_menu srm on usr.id = srm.role_id left join sys_menu u on srm.menu_id = u.id where t.user_id = $1"#;
                match db.query_all(Statement::from_sql_and_values(DatabaseBackend::MySql, sql_str, [id.into()])).await {
                    Ok(qr) => {
                        for x in qr {
                            btn_menu.push(x.try_get("", "api_url").unwrap_or_default());
                        }
                        log::info!("ordinary login: {:?}",id);
                        btn_menu
                    }
                    Err(err) => {
                        error!("err: {:?}",err.to_string());
                        btn_menu
                    }
                }
            }
        }
        Err(err) => {
            error!("err: {:?}",err.to_string());
            btn_menu
        }
    }
}

#[post("/query_user_role", data = "<item>")]
pub async fn query_user_role(db: &State<DatabaseConnection>, item: Json<QueryUserRoleReq>, _auth: Token) -> Value {
    log::info!("query_user_role params: {:?}", item);
    let db = db as &DatabaseConnection;
    let mut user_role_ids: Vec<i64> = Vec::new();

    for x in SysUserRole::find().filter(sys_user_role::Column::UserId.eq(item.user_id.clone())).all(db).await.unwrap() {
        user_role_ids.push(x.role_id);
    }

    let mut sys_role_list: Vec<UserRoleList> = Vec::new();

    for x in SysRole::find().all(db).await.unwrap() {
        sys_role_list.push(UserRoleList {
            id: x.id,
            status_id: x.status_id,
            sort: x.sort,
            role_name: x.role_name,
            remark: x.remark,
            create_time: x.create_time.to_string(),
            update_time: x.update_time.to_string(),
        });
    }

    json!(ok_result_data(QueryUserRoleData {
        sys_role_list,
        user_role_ids,
    }))
}

#[post("/update_user_role", data = "<item>")]
pub async fn update_user_role(db: &State<DatabaseConnection>, item: Json<UpdateUserRoleReq>, _auth: Token) -> Value {
    log::info!("update_user_role params: {:?}", item);
    let db = db as &DatabaseConnection;

    let user_role = item.0;
    let user_id = user_role.user_id;
    let role_ids = &user_role.role_ids;

    if user_id == 1 {
        return json!(err_result_msg("不能修改超级管理员的角色".to_string()));
    }

    let result = SysUserRole::delete_many().filter(sys_user_role::Column::UserId.eq(user_id)).exec(db).await;

    if result.is_err() {
        return json!(err_result_msg("更新用户角色异常".to_string()));
    }

    let mut sys_role_user_list: Vec<sys_user_role::ActiveModel> = Vec::new();
    for role_id in role_ids {
        let r_id = role_id.clone();
        sys_role_user_list.push(sys_user_role::ActiveModel {
            id: NotSet,
            status_id: Set(1),
            sort: Set(1),
            role_id: Set(r_id),
            user_id: Set(user_id.clone()),
            ..Default::default()
        })
    }

    json!(&handle_result(SysUserRole::insert_many(sys_role_user_list).exec(db).await))
}

#[get("/query_user_menu")]
pub async fn query_user_menu(db: &State<DatabaseConnection>, auth: Token) -> Value {
    log::info!("query_user_menu params: {:?}", auth);
    let db = db as &DatabaseConnection;

    match SysUser::find_by_id(auth.id.clone()).one(db).await {
        Ok(sys_user) => {
            match sys_user {
                None => {
                    json!(err_result_msg("用户不存在".to_string()))
                }
                Some(user) => {
                    match SysUserRole::find().filter(sys_user_role::Column::UserId.eq(user.id)).filter(sys_user_role::Column::RoleId.eq(1)).one(db).await {
                        Ok(opt_user) => {
                            let sys_menu_list: Vec<sys_menu::Model>;

                            if let Some(u) = opt_user {
                                sys_menu_list = SysMenu::find().all(db).await.unwrap();
                            } else {
                                let sql_str = r#"select u.* from sys_user_role t left join sys_role usr on t.role_id = usr.id left join sys_role_menu srm on usr.id = srm.role_id left join sys_menu u on srm.menu_id = u.id where t.user_id = $1 order by u.id asc"#;
                                sys_menu_list = SysMenu::find().from_raw_sql(Statement::from_sql_and_values(DatabaseBackend::MySql, sql_str, [user.id.clone().into()])).all(db).await.unwrap();
                            }
                            let mut btn_menu: Vec<String> = Vec::new();
                            let mut sys_menu_ids: Vec<i64> = Vec::new();

                            for x in sys_menu_list {
                                if x.menu_type != 3 {
                                    sys_menu_ids.push(x.id);
                                    sys_menu_ids.push(x.parent_id)
                                }

                                btn_menu.push(x.api_url);
                            }

                            let mut sys_menu: Vec<MenuUserList> = Vec::new();
                            for y in SysMenu::find().filter(sys_menu::Column::Id.is_in(sys_menu_ids)).order_by_asc(sys_menu::Column::Sort).all(db).await.unwrap() {
                                sys_menu.push(MenuUserList {
                                    id: y.id,
                                    parent_id: y.parent_id,
                                    name: y.menu_name,
                                    icon: y.menu_icon.unwrap_or_default(),
                                    api_url: y.api_url,
                                    menu_type: y.menu_type,
                                    path: y.menu_url,
                                });
                            }

                            json!(ok_result_data(QueryUserMenuData {
                                    sys_menu,
                                    btn_menu,
                                    avatar: "https://gw.alipayobjects.com/zos/antfincdn/XAosXuNZyF/BiazfanxmamNRoxxVxka.png".to_string(),
                                    name: user.user_name,
                                }))
                        }
                        Err(err) => {
                            json!(err_result_msg(err.to_string()))
                        }
                    }
                }
            }
        }
        // 查询用户数据库异常
        Err(err) => {
            json!(err_result_msg(err.to_string()))
        }
    }
}

// 查询用户列表
#[post("/user_list", data = "<item>")]
pub async fn user_list(db: &State<DatabaseConnection>, item: Json<UserListReq>, _auth: Token) -> Value {
    log::info!("query user_list params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let paginator = SysUser::find()
        .apply_if(item.mobile.clone(), |mut query, v| {
            query.filter(sys_user::Column::Mobile.eq(v))
        })
        .apply_if(item.status_id.clone(), |mut query, v| {
            query.filter(sys_user::Column::StatusId.eq(v))
        }).paginate(db, item.page_size.clone());

    let total = paginator.num_items().await.unwrap_or_default();
    let result = paginator.fetch_page(item.page_no.clone() - 1).await;

    match result {
        Ok(page) => {
            let mut list_data: Vec<UserListData> = Vec::new();

            for user in page {
                list_data.push(UserListData {
                    id: user.id,
                    sort: user.sort,
                    status_id: user.status_id,
                    mobile: user.mobile,
                    user_name: user.user_name,
                    remark: user.remark.unwrap_or_default(),
                    create_time: user.create_time.to_string(),
                    update_time: user.update_time.to_string(),
                })
            }

            json!(ok_result_page(list_data, total))
        }
        Err(err) => {
            json!(err_result_page(err.to_string()))
        }
    }
}

// 添加用户信息
#[post("/user_save", data = "<item>")]
pub async fn user_save(db: &State<DatabaseConnection>, item: Json<UserSaveReq>, _auth: Token) -> Value {
    log::info!("user_save params: {:?}", &item);

    let user = item.0;
    let db = db as &DatabaseConnection;

    let sys_user = sys_user::ActiveModel {
        id: NotSet,
        status_id: Set(user.status_id),
        sort: Set(user.sort),
        mobile: Set(user.mobile),
        user_name: Set(user.user_name),
        remark: Set(user.remark),
        ..Default::default()
    };

    json!(&handle_result(SysUser::insert(sys_user).exec(db).await))
}

// 更新用户信息
#[post("/user_update", data = "<item>")]
pub async fn user_update(db: &State<DatabaseConnection>, item: Json<UserUpdateReq>, _auth: Token) -> Value {
    log::info!("user_update params: {:?}", &item);

    let user = item.0;
    let db = db as &DatabaseConnection;

    match SysUser::find_by_id(user.id.clone()).one(db).await {
        Ok(opt_user) => {
            if let Some(u) = opt_user {
                let sys_user = sys_user::ActiveModel {
                    id: Set(user.id),
                    status_id: Set(user.status_id),
                    sort: Set(user.sort),
                    mobile: Set(user.mobile),
                    user_name: Set(user.user_name),
                    remark: Set(user.remark),
                    ..Default::default()
                };

                json!(&handle_result(SysUser::update(sys_user).exec(db).await))
            } else {
                json!(err_result_msg("用户不存在".to_string()))
            }
        }
        Err(err) => {
            json!(err_result_msg(err.to_string()))
        }
    }
}

// 删除用户信息
#[post("/user_delete", data = "<item>")]
pub async fn user_delete(db: &State<DatabaseConnection>, item: Json<UserDeleteReq>, _auth: Token) -> Value {
    log::info!("user_delete params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let ids = item.ids.clone();
    for id in ids {
        if id != 1 {//id为1的用户为系统预留用户,不能删除
            let _ = SysUser::delete_by_id(id).exec(db).await;
        }
    }

    json!(ok_result_msg("删除用户信息成功".to_string()))
}

// 更新用户密码
#[post("/update_user_password", data = "<item>")]
pub async fn update_user_password(db: &State<DatabaseConnection>, item: Json<UpdateUserPwdReq>, _auth: Token) -> Value {
    log::info!("update_user_pwd params: {:?}", &item);
    let db = db as &DatabaseConnection;
    let user_pwd = item.0;

    match SysUser::find_by_id(user_pwd.id).one(db).await {
        Ok(user_result) => {
            match user_result {
                None => {
                    json!(err_result_msg("用户不存在".to_string()))
                }
                Some(mut user) => {
                    if user.password == user_pwd.pwd {
                        let mut s_user: sys_user::ActiveModel = user.into();
                        s_user.password = Set(user_pwd.re_pwd);

                        json!(&handle_result(s_user.update(db).await))
                    } else {
                        json!(err_result_msg("旧密码不正确".to_string()))
                    }
                }
            }
        }
        Err(err) => {
            json!(err_result_msg(err.to_string()))
        }
    }
}
