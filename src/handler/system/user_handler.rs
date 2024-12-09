use rocket::serde::json::{serde_json, Json, Value};
use rocket::State;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, EntityTrait, NotSet, PaginatorTrait, QueryFilter, QueryOrder, QueryTrait, Statement};
use sea_orm::ActiveValue::Set;
use crate::common::error::WhoUnfollowedError;
use crate::model::system::prelude::{SysMenu, SysRole, SysUser, SysUserRole};
use crate::middleware::auth::Token;
use crate::utils::jwt_util::JWTToken;
use crate::common::error_handler::ErrorResponder;
use crate::common::result::BaseResponse;
use crate::model::system::{sys_menu, sys_user, sys_user_role};
use crate::vo::system::user_vo::*;

// 后台用户登录
#[post("/login", data = "<item>")]
pub async fn login(db: &State<DatabaseConnection>, item: Json<UserLoginReq>) -> Result<Value, ErrorResponder> {
    log::info!("user login params: {:?}", &item);

    let db = db as &DatabaseConnection;

    let user_result = SysUser::find().filter(sys_user::Column::Mobile.eq(&item.mobile)).one(db).await?;
    log::info!("select_by_mobile: {:?}",user_result);

    if user_result.is_none() {
        return Ok(BaseResponse::<String>::err_result_msg("用户不存在!".to_string()));
    }

    let user = user_result.unwrap();

    let id = user.id;
    let username = user.user_name;
    let password = user.password;

    if password.ne(&item.password) {
        return Ok(BaseResponse::<String>::err_result_msg("密码不正确!".to_string()));
    }

    let btn_menu = query_btn_menu(db, id.clone()).await?;

    if btn_menu.len() == 0 {
        return Ok(BaseResponse::<String>::err_result_msg("用户没有分配角色或者菜单,不能登录!".to_string()));
    }

    match JWTToken::new(id, &username, btn_menu).create_token("123") {
        Ok(token) => {
            Ok(BaseResponse::<String>::ok_result_data(token))
        }
        Err(err) => {
            let er = match err {
                WhoUnfollowedError::JwtTokenError(s) => { s }
                _ => "no math error".to_string()
            };

            Ok(BaseResponse::<String>::err_result_msg(er))
        }
    }
}

// 登录的时候 查询权限
async fn query_btn_menu(db: &DatabaseConnection, id: i64) -> Result<Vec<String>, ErrorResponder> {
    let mut btn_menu: Vec<String> = Vec::new();
    //角色Id为1的是系统预留超级管理员角色
    if SysUserRole::find().filter(sys_user_role::Column::UserId.eq(id.clone())).filter(sys_user_role::Column::RoleId.eq(1)).count(db).await? != 0 {
        for x in SysMenu::find().all(db).await? {
            btn_menu.push(x.api_url);
        }
        log::info!("admin login: {:?}",id);
    } else {
        let sql_str = r#"select distinct u.api_url from sys_user_role t left join sys_role usr on t.role_id = usr.id left join sys_role_menu srm on usr.id = srm.role_id left join sys_menu u on srm.menu_id = u.id where t.user_id = $1"#;
        for x in db.query_all(Statement::from_sql_and_values(DatabaseBackend::MySql, sql_str, [id.into()])).await? {
            btn_menu.push(x.try_get("", "api_url").unwrap_or_default());
        }
        log::info!("ordinary login: {:?}",id);
    }

    Ok(btn_menu)
}

#[post("/query_user_role", data = "<item>")]
pub async fn query_user_role(db: &State<DatabaseConnection>, item: Json<QueryUserRoleReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("query_user_role params: {:?}", item);
    let db = db as &DatabaseConnection;
    let mut user_role_ids: Vec<i64> = Vec::new();

    for x in SysUserRole::find().filter(sys_user_role::Column::UserId.eq(item.user_id.clone())).all(db).await? {
        user_role_ids.push(x.role_id);
    }

    let mut sys_role_list: Vec<UserRoleList> = Vec::new();

    for x in SysRole::find().all(db).await? {
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

    Ok(BaseResponse::<QueryUserRoleData>::ok_result_data(QueryUserRoleData {
        sys_role_list,
        user_role_ids,
    }))
}

// 更新用户角色信息
#[post("/update_user_role", data = "<item>")]
pub async fn update_user_role(db: &State<DatabaseConnection>, item: Json<UpdateUserRoleReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("update_user_role params: {:?}", item);
    let db = db as &DatabaseConnection;

    let user_role = item.0;
    let user_id = user_role.user_id;
    let role_ids = &user_role.role_ids;

    if user_id == 1 {
        return Ok(BaseResponse::<String>::err_result_msg("不能修改超级管理员的角色!".to_string()));
    }

    SysUserRole::delete_many().filter(sys_user_role::Column::UserId.eq(user_id)).exec(db).await?;

    let mut sys_role_user_list: Vec<sys_user_role::ActiveModel> = Vec::new();
    for role_id in role_ids {
        let r_id = role_id.clone();
        if r_id == 1 {
            continue;
        }
        sys_role_user_list.push(sys_user_role::ActiveModel {
            id: NotSet,
            status_id: Set(1),
            sort: Set(1),
            role_id: Set(r_id),
            user_id: Set(user_id.clone()),
            ..Default::default()
        })
    }

    SysUserRole::insert_many(sys_role_user_list).exec(db).await?;
    Ok(BaseResponse::<String>::ok_result())
}

#[get("/query_user_menu")]
pub async fn query_user_menu(db: &State<DatabaseConnection>, auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("query_user_menu params: {:?}", auth);
    let db = db as &DatabaseConnection;

    if SysUser::find_by_id(auth.id.clone()).one(db).await?.is_none() {
        return Ok(BaseResponse::<String>::err_result_msg("用户不存在".to_string()));
    }

    let sys_menu_list: Vec<sys_menu::Model>;

    if SysUserRole::find().filter(sys_user_role::Column::UserId.eq(auth.id.clone())).filter(sys_user_role::Column::RoleId.eq(1)).one(db).await?.is_some() {
        sys_menu_list = SysMenu::find().all(db).await?;
    } else {
        let sql_str = r#"select u.* from sys_user_role t left join sys_role usr on t.role_id = usr.id left join sys_role_menu srm on usr.id = srm.role_id left join sys_menu u on srm.menu_id = u.id where t.user_id = $1 order by u.id asc"#;
        sys_menu_list = SysMenu::find().from_raw_sql(Statement::from_sql_and_values(DatabaseBackend::MySql, sql_str, [auth.id.clone().clone().into()])).all(db).await?;
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
    for y in SysMenu::find().filter(sys_menu::Column::Id.is_in(sys_menu_ids)).filter(sys_menu::Column::StatusId.eq(1)).order_by_asc(sys_menu::Column::Sort).all(db).await? {
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

    let avatar = "https://gw.alipayobjects.com/zos/antfincdn/XAosXuNZyF/BiazfanxmamNRoxxVxka.png".to_string();

    let resp = QueryUserMenuData {
        sys_menu,
        btn_menu,
        avatar: avatar,
        name: auth.username.clone(),
    };

    let r = serde_json::to_string(&resp).unwrap();
    Ok(BaseResponse::<String>::ok_result_data(r))
}

// 查询用户列表
#[post("/user_list", data = "<item>")]
pub async fn user_list(db: &State<DatabaseConnection>, item: Json<UserListReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("query user_list params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let paginator = SysUser::find()
        .apply_if(item.mobile.clone(), |query, v| {
            query.filter(sys_user::Column::Mobile.eq(v))
        })
        .apply_if(item.status_id.clone(), |query, v| {
            query.filter(sys_user::Column::StatusId.eq(v))
        }).paginate(db, item.page_size.clone());

    let total = paginator.num_items().await.unwrap_or_default();

    let mut list_data: Vec<UserListData> = Vec::new();

    for user in paginator.fetch_page(item.page_no.clone() - 1).await? {
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

    Ok(BaseResponse::<Vec<UserListData>>::ok_result_page(list_data, total))
}

// 添加用户信息
#[post("/user_save", data = "<item>")]
pub async fn user_save(db: &State<DatabaseConnection>, item: Json<UserSaveReq>, _auth: Token) -> Result<Value, ErrorResponder> {
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

    SysUser::insert(sys_user).exec(db).await?;
    Ok(BaseResponse::<String>::ok_result())
}

// 更新用户信息
#[post("/user_update", data = "<item>")]
pub async fn user_update(db: &State<DatabaseConnection>, item: Json<UserUpdateReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("user_update params: {:?}", &item);

    let user = item.0;
    let db = db as &DatabaseConnection;

    if SysUser::find_by_id(user.id.clone()).one(db).await?.is_none() {
        // return Ok(BaseResponse::<String>::err_result_msg("用户不存在!")));
        return Err(ErrorResponder::from("用户不存在!"));
    }

    let sys_user = sys_user::ActiveModel {
        id: Set(user.id),
        status_id: Set(user.status_id),
        sort: Set(user.sort),
        mobile: Set(user.mobile),
        user_name: Set(user.user_name),
        remark: Set(user.remark),
        ..Default::default()
    };

    SysUser::update(sys_user).exec(db).await?;
    Ok(BaseResponse::<String>::ok_result())
}

// 删除用户信息
#[post("/user_delete", data = "<item>")]
pub async fn user_delete(db: &State<DatabaseConnection>, item: Json<UserDeleteReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("user_delete params: {:?}", &item);
    let db = db as &DatabaseConnection;

    let ids = item.ids.clone();
    for id in ids {
        if id != 1 { //id为1的用户为系统预留用户,不能删除
            let _ = SysUser::delete_by_id(id).exec(db).await;
        }
    }

    Ok(BaseResponse::<String>::ok_result())
}

// 更新用户密码
#[post("/update_user_password", data = "<item>")]
pub async fn update_user_password(db: &State<DatabaseConnection>, item: Json<UpdateUserPwdReq>, _auth: Token) -> Result<Value, ErrorResponder> {
    log::info!("update_user_pwd params: {:?}", &item);
    let db = db as &DatabaseConnection;
    let user_pwd = item.0;

    let result = SysUser::find_by_id(user_pwd.id).one(db).await?;
    if result.is_none() {
        return Ok(BaseResponse::<String>::err_result_msg("用户不存在!".to_string()))
    };

    let user = result.unwrap();
    if user.password == user_pwd.pwd {
        let mut s_user: sys_user::ActiveModel = user.into();
        s_user.password = Set(user_pwd.re_pwd);

        s_user.update(db).await?;
        Ok(BaseResponse::<String>::ok_result())
    } else {
        Ok(BaseResponse::<String>::err_result_msg("旧密码不正确!".to_string()))
    }
}
