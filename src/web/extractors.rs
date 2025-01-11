use salvo::{
    extract::Metadata,
    http::{header::CONTENT_TYPE, mime::APPLICATION_JSON},
};
use serde::Deserialize;

use super::resp::Response;

pub struct Json<T>(pub T);

impl<'ex, T> salvo::Extractible<'ex> for Json<T>
where
    T: Deserialize<'ex>,
{
    fn metadata() -> &'ex Metadata {
        static METADATA: Metadata = Metadata::new("");
        &METADATA
    }

    async fn extract(
        req: &'ex mut salvo::Request,
    ) -> Result<Self, impl salvo::Writer + Send + std::fmt::Debug + 'static>
    where
        Self: Sized,
    {
        if req
            .content_type()
            .and_then(|mime| (mime == APPLICATION_JSON).then(|| ()))
            .is_none()
        {
            return Err(Response::custom(415, "request content_type is not json"));
        }
        match sonic_rs::from_slice(&*req.payload().await?) {
            Ok(json) => Ok(Json(json)),
            Err(err) => Err(Response::custom(
                400,
                format!("invalid request body: {err}"),
            )),
        }
    }
}
