use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use crate::utils::jwt_util::JWTToken;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct Token;


#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = ();
    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let path = request.uri().to_string();
        let header_auth = request.headers().get_one("Authorization");
        if let Some(header_auth) = header_auth {
            let split_vec = header_auth.split_whitespace().collect::<Vec<_>>();
            if split_vec.len() == 2 && split_vec[0] == "Bearer" {
                let token = split_vec[1];
                let jwt_token_e = JWTToken::verify("123", &token);
                let jwt_token = match jwt_token_e {
                    Ok(data) => { data }
                    Err(err) => {
                        log::error!("check token fail path: {}, token: {}, err: {}", path, token, err.to_string());
                        return Outcome::Failure((Status::Unauthorized, ()));
                    }
                };

                let mut flag: bool = false;
                for token_permission in &jwt_token.permissions {
                    if token_permission.to_string() == path {
                        flag = true;
                        break;
                    }
                }
                return if flag {
                    Outcome::Success(Token)
                } else {
                    log::error!("no permissions path: {}, token: {}", path, token);
                    Outcome::Failure((Status::Unauthorized, ()))
                };
            }
            log::error!("token format wrong path: {}", path);
        }
        log::error!("Authorization miss path: {}", path);
        Outcome::Failure((Status::Unauthorized, ()))
    }
}
