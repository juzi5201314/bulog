use std::time::Duration;

use resp::Response;
use salvo::{
    Depot, FlowCtrl, Listener, Request, Router, Server, affix_state,
    conn::TcpListener,
    handler,
    server::ServerHandle,
    session::{CookieStore, SessionHandler},
};
use surrealdb::{Surreal, engine::any::Any};
use tokio::task::JoinHandle;

use crate::db::{self, model::config::is_new_install};

mod extractors;
mod resp;
mod v1;

struct Installed;

pub async fn web_server() -> anyhow::Result<(ServerHandle, JoinHandle<()>)> {
    let bind = std::env::var("BU_BIND").unwrap_or_else(|_| "0.0.0.0:8686".to_owned());
    let db = db::db(None).await?;
    let session_handler = session_secret(&db).await?;
    let router = Router::new().hoop(session_handler).push(router(db));

    tracing::info!("listen on {}", bind);

    let listener = TcpListener::new(bind).bind().await;
    let server = Server::new(listener);

    let server_handle = server.handle();
    let join_handle = tokio::spawn(server.serve(router));
    Ok((server_handle, join_handle))
}

pub(crate) fn router(db: Surreal<Any>) -> Router {
    Router::new()
        .hoop(affix_state::inject(db))
        .hoop(initialization_check)
        .push(v1::router())
}

#[handler]
async fn initialization_check(
    ctrl: &mut FlowCtrl,
    req: &mut Request,
    resp: &mut salvo::Response,
    depot: &mut Depot,
) -> anyhow::Result<()> {
    if !depot.contains::<Installed>() {
        let db = depot.obtain::<Surreal<Any>>().unwrap();
        let installed = !is_new_install(db).await?;

        if !installed && !req.uri().path().ends_with("/install") {
            resp.render(Response::custom(0, "uninitialized"));
            ctrl.cease();
        } else if installed {
            depot.inject(Installed);
        }
    }
    Ok(())
}

async fn session_secret(db: &Surreal<Any>) -> anyhow::Result<SessionHandler<CookieStore>> {
    let secret = db
        .query(
            "LET $secret = (SELECT session FROM secret:bulog); \
        IF $secret != null { \
            RETURN $secret; \
        } ELSE { \
            LET $secret = rand::string(32); \
            CREATE secret:bulog SET session = $secret; \
            RETURN $secret; \
        }",
        )
        .await?
        .take::<Option<String>>(0)?
        .unwrap();
    SessionHandler::builder(CookieStore::new(), secret.as_bytes())
        .cookie_name("bulog")
        .session_ttl(Some(Duration::from_secs(30 * 3600 * 24)))
        .build()
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use salvo::{
        Service,
        http::{header::CONTENT_TYPE, mime},
        test::{ResponseExt, TestClient},
    };
    use serde::Deserialize;
    use serde_json::json;

    use crate::db::{db, test_db};

    macro_rules! get {
        ($target:literal @$service:expr) => {
            TestClient::get(concat!("http://localhost:0/", $target))
                .send(&$service)
                .await
                .take_json::<Response>()
                .await
                .unwrap()
        };
        ($target:literal @$service:expr => text) => {
            TestClient::get(concat!("http://localhost:0/", $target))
                .send(&$service)
                .await
                .take_string()
                .await
                .unwrap()
        };
    }

    macro_rules! post {
        ($target:literal @$service:expr, $body:expr) => {
            TestClient::post(concat!("http://localhost:0/", $target))
                .add_header(CONTENT_TYPE, mime::APPLICATION_JSON.essence_str(), true)
                .json($body)
                .send(&$service)
                .await
                .take_json::<Response>()
                .await
                .unwrap()
        };
        ($target:literal @$service:expr, $body:expr => text) => {
            TestClient::post(concat!("http://localhost:0/", $target))
                .add_header(CONTENT_TYPE, mime::JSON.as_str(), true)
                .json($body)
                .send(&$service)
                .await
                .take_string()
                .await
                .unwrap()
        };
    }

    #[derive(Deserialize)]
    struct Response {
        code: u16,
        message: String,
        data: serde_json::Value,
    }

    async fn service() -> Service {
        Service::new(super::router(test_db().await.unwrap()))
    }

    #[tokio::test]
    async fn test_install() {
        let service = Service::new(super::router(db(Some("mem://".to_owned())).await.unwrap()));
        let notinstalled = get!("/v1/config"@service);
        assert_eq!(notinstalled.code, 0);
        assert_eq!(notinstalled.message, "uninitialized");

        let install = post!("/v1/install"@service, &json!({
            "title": "new blog",
            "description": "an apple",
            "password": "$test$"
        }));
        assert_eq!(install.code, 200);

        let installed = get!("/v1/config"@service);
        assert_eq!(installed.code, 200);
        assert_eq!(installed.message, "");
        assert_eq!(installed.data["title"], "new blog");
        assert_eq!(installed.data["description"], "an apple");
        assert_eq!(installed.data["password"], "");
    }
}
