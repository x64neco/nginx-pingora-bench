use async_trait::async_trait;
use module_utils::RequestFilter;
use pingora_core::server::{configuration::ServerConf, Server};
use pingora_core::upstreams::peer::HttpPeer;
use pingora_core::{Error, ErrorType, Result};
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};
use static_files_module::{StaticFilesConf, StaticFilesHandler};

struct StaticServer(StaticFilesHandler);

#[async_trait]
impl ProxyHttp for StaticServer {
    type CTX = <StaticFilesHandler as RequestFilter>::CTX;

    fn new_ctx(&self) -> Self::CTX {
        StaticFilesHandler::new_ctx()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        session.set_keepalive(Some(60));
        self.0.handle(session, ctx).await
    }

    async fn upstream_peer(&self, _: &mut Session, _: &mut Self::CTX) -> Result<Box<HttpPeer>> {
        Err(Error::new(ErrorType::HTTPStatus(404)))
    }
}

fn main() {
    let threads: usize = std::env::var("WORKERS")
        .unwrap_or_else(|_| "1".into())
        .parse()
        .expect("WORKERS must be a positive integer");
    assert!(threads > 0, "WORKERS must be positive");
    let root = std::env::var("STATIC_ROOT").unwrap_or_else(|_| "./www".into());
    let root = std::fs::canonicalize(root).expect("create ./www or set STATIC_ROOT to a directory");
    assert!(
        std::path::Path::new(&root).is_dir(),
        "STATIC_ROOT must exist"
    );
    let handler = StaticFilesConf {
        root: Some(root.into()),
        ..Default::default()
    }
    .try_into()
    .expect("invalid static files configuration");
    let conf = ServerConf {
        threads,
        grace_period_seconds: Some(0),
        graceful_shutdown_timeout_seconds: Some(1),
        ..Default::default()
    };
    let mut server = Server::new_with_opt_and_conf(Default::default(), conf);
    server.bootstrap();
    let listen = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let mut service = http_proxy_service(&server.configuration, StaticServer(handler));
    service.add_tcp(&listen);
    server.add_service(service);
    eprintln!("Pingora 0.2.0 + static-files-module 0.2.0; http://{listen}; workers={threads}");
    server.run_forever();
}
