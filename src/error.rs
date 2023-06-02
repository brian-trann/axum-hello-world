use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;
#[derive(Clone, Serialize, Debug, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // you may wnat to split this in the future
    LoginFail,
    // model errors
    TicketDeleteFailIdNotFound { id: u64 },
    // auth errors
    AuthFailNoTokenCookie,
    AuthFailTokenWrongFormat,
    AuthFailedCtxNotInRequestExt,
}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // the will know how to convert client <->server error
        println!("->> {:<12} - {self:?}", "INTO_RES");
        // create placeholder axum res
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        //insert error into res
        response.extensions_mut().insert(self);
        response
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        // either be exhaustive, or have fallback
        #[allow(unreachable_patterns)]
        match self {
            Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),
            // auth
            Self::AuthFailNoTokenCookie
            | Self::AuthFailTokenWrongFormat
            | Self::AuthFailedCtxNotInRequestExt => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // model
            Self::TicketDeleteFailIdNotFound { .. } => {
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }

            // fallback
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}
#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    INVALID_PARAMS,
    SERVICE_ERROR,
}
