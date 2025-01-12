use salvo::http::{header::CONTENT_TYPE, mime::APPLICATION_JSON};
use serde::Serialize;

pub type RespResult<T> = Result<Response<T>, Response<()>>;

#[derive(Debug, Serialize)]
pub struct Response<T> {
    pub code: u16,
    pub message: String,
    pub data: T,
}

impl Response<()> {
    pub fn empty() -> Response<()> {
        Response {
            code: 200,
            message: String::new(),
            data: (),
        }
    }

    pub fn error(msg: impl Into<String>) -> Response<()> {
        Response {
            code: 500,
            message: msg.into(),
            data: (),
        }
    }

    pub fn custom(code: u16, msg: impl Into<String>) -> Response<()> {
        Response {
            code,
            message: msg.into(),
            data: (),
        }
    }
}

impl<T> Response<T>
where
    T: Serialize,
{
    pub fn ok(value: T) -> Response<T> {
        Response {
            code: 200,
            message: String::new(),
            data: value,
        }
    }
}

impl<E> From<E> for Response<()>
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Response::error(err.into().to_string())
    }
}

impl<T> salvo::Scribe for Response<T>
where
    T: Serialize,
{
    fn render(self, res: &mut salvo::Response) {
        res.add_header(CONTENT_TYPE, APPLICATION_JSON.essence_str(), true)
            .unwrap();
        match sonic_rs::to_vec(&self) {
            Ok(bytes) => res.write_body(bytes).unwrap(),
            Err(err) => res.render(format!(
                r#"{{"code": 500, "message": "json encode error: {err}", "data": {{}}}}"#,
            )),
        }
    }
}
