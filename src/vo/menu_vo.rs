use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MenuListReq {
    pub menu_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct MenuListResp {
    pub msg: String,
    pub code: i32,
    pub total: u64,
    pub data: Option<Vec<MenuListData>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MenuListData {
    pub id: i32,
    pub sort: i32,
    pub status_id: i32,
    pub parent_id: i32,
    pub menu_name: String,
    pub label: String,
    pub menu_url: String,
    pub icon: String,
    pub api_url: String,
    pub remark: String,
    pub menu_type: i32,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MenuSaveReq {
    pub sort: i32,
    pub status_id: i32,
    pub parent_id: Option<i32>,
    pub menu_name: String,
    pub menu_url: String,
    pub icon: String,
    pub api_url: String,
    pub remark: String,
    pub menu_type: i32,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MenuUpdateReq {
    pub id: i32,
    pub sort: i32,
    pub status_id: i32,
    pub parent_id: i32,
    pub menu_name: String,
    pub menu_url: String,
    pub icon: String,
    pub api_url: String,
    pub remark: String,
    pub menu_type: i32,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MenuDeleteReq {
    pub ids: Vec<i32>,
}
