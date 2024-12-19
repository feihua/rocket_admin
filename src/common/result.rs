use rocket::serde::json::serde_json::json;
use rocket::serde::json::Value;
use serde::Serialize;
use std::fmt::Debug;
// 统一返回vo
#[derive(Serialize, Debug, Clone)]
pub struct BaseResponse<T>
where
    T: Serialize + Debug,
{
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ResponsePage<T>
where
    T: Serialize + Debug,
{
    pub code: i32,
    pub msg: String,
    pub total: u64,
    pub success: bool,
    pub data: Option<T>,
}

impl<T> BaseResponse<T>
where
    T: Serialize + Debug + Send,
{
    pub fn ok_result() -> Value {
        json!(BaseResponse {
            msg: "操作成功".to_string(),
            code: 0,
            data: Some("None".to_string()),
        })
    }

    pub fn ok_result_msg(msg: String) -> Value {
        json!(BaseResponse {
            msg: msg.to_string(),
            code: 0,
            data: Some("None".to_string()),
        })
    }

    pub fn ok_result_code(code: i32, msg: String) -> Value {
        json!(BaseResponse {
            msg: msg.to_string(),
            code,
            data: Some("None".to_string()),
        })
    }

    pub fn ok_result_data(data: T) -> Value {
        json!(BaseResponse {
            msg: "操作成功".to_string(),
            code: 0,
            data: Some(data),
        })
    }
    pub fn err_result_data(data: T, msg: String) -> Value {
        json!(BaseResponse {
            msg,
            code: 1,
            data: Some(data),
        })
    }

    pub fn err_result_msg(msg: String) -> Value {
        json!(BaseResponse {
            msg: msg.to_string(),
            code: 1,
            data: Some("None".to_string()),
        })
    }

    pub fn err_result_code(code: i32, msg: String) -> Value {
        json!(BaseResponse {
            msg: msg.to_string(),
            code,
            data: Some("None".to_string()),
        })
    }

    pub fn ok_result_page(data: T, total: u64) -> Value {
        json!(ResponsePage {
            msg: "操作成功".to_string(),
            code: 0,
            success: true,
            data: Some(data),
            total,
        })
    }
}
