use std::time::Duration;

use resp::Response;
use salvo::{
    Depot, FlowCtrl, Listener, Request, Router, Server, Service, affix_state,
    catcher::Catcher,
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
    let router = router(db).await?;

    tracing::info!("listen on {}", bind);

    let listener = TcpListener::new(bind).bind().await;
    let server = Server::new(listener);

    let server_handle = server.handle();
    let join_handle = tokio::spawn(server.serve(Service::new(router).catcher(catcher())));
    Ok((server_handle, join_handle))
}

pub(crate) async fn router(db: Surreal<Any>) -> anyhow::Result<Router> {
    let session_handler = session_secret(&db).await?;
    Ok(Router::new()
        .hoop(session_handler)
        .hoop(affix_state::inject(db))
        .hoop(initialization_check)
        .push(v1::router()))
}

pub(crate) fn catcher() -> Catcher {
    Catcher::default().hoop(v1::catch404)
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
            "LET $secret = (SELECT session FROM ONLY secret:bulog).session; \
            IF $secret != NONE { \
                RETURN $secret; \
            } ELSE { \
                LET $secret = rand::string(64); \
                CREATE secret:bulog SET session = $secret; \
                RETURN $secret; \
            }",
        )
        .await?
        .take::<Option<String>>(1)?
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
        http::{
            cookie::CookieJar,
            header::{CONTENT_TYPE, COOKIE},
            mime,
        },
        test::{ResponseExt, TestClient},
    };
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::db::{db, test_db};

    use super::catcher;

    struct HttpClient {
        service: Service,
        cookie: CookieJar,
    }

    impl HttpClient {
        pub async fn get(&self, uri: &str) -> Response {
            TestClient::get(format!("http://localhost:0/{}", uri))
                .add_header(
                    COOKIE,
                    &self
                        .cookie
                        .iter()
                        .map(|c| c.encoded().to_string())
                        .collect::<Vec<_>>()
                        .join("; "),
                    true,
                )
                .send(&self.service)
                .await
                .take_json()
                .await
                .unwrap()
        }

        pub async fn post<T>(&mut self, uri: &str, data: &T) -> Response
        where
            T: Serialize,
        {
            let mut resp = TestClient::post(format!("http://localhost:0/{}", uri))
                .add_header(CONTENT_TYPE, mime::APPLICATION_JSON.essence_str(), true)
                .json(data)
                .send(&self.service)
                .await;
            for cookie in resp.cookies().iter() {
                self.cookie.add(cookie.clone());
                println!("{:?}", cookie);
            }
            resp.take_json().await.unwrap()
        }

        pub fn new(service: Service) -> Self {
            Self {
                service,
                cookie: CookieJar::default(),
            }
        }

        pub async fn default() -> Self {
            Self::new(service().await)
        }
    }

    #[derive(Deserialize)]
    struct Response {
        code: u16,
        message: String,
        data: serde_json::Value,
    }

    async fn service() -> Service {
        Service::new(super::router(test_db().await.unwrap()).await.unwrap()).catcher(catcher())
    }

    #[tokio::test]
    async fn test_install() {
        let service = Service::new(
            super::router(db(Some("mem://".to_owned())).await.unwrap())
                .await
                .unwrap(),
        );
        let mut client = HttpClient::new(service);
        let notinstalled = client.get("/v1/config").await;
        assert_eq!(notinstalled.code, 0);
        assert_eq!(notinstalled.message, "uninitialized");

        let install = client
            .post(
                "/v1/install",
                &json!({
                    "title": "new blog",
                    "description": "an apple",
                    "password": "$test$"
                }),
            )
            .await;
        assert_eq!(install.code, 200);

        let installed = client.get("/v1/config").await;
        assert_eq!(installed.code, 200);
        assert_eq!(installed.message, "");
        assert_eq!(installed.data["title"], "new blog");
        assert_eq!(installed.data["description"], "an apple");
        assert_eq!(installed.data["password"], "");
    }

    #[tokio::test]
    async fn test_login() {
        let mut client = HttpClient::default().await;
        let logged = client.get("/v1/login").await;
        assert_eq!(logged.code, 403);

        let resp = client
            .post(
                "/v1/login",
                &json!({
                    "password": ""
                }),
            )
            .await;
        assert_eq!(resp.code, 200);

        let logged = client.get("/v1/login").await;
        assert_eq!(logged.code, 200);
    }
}
