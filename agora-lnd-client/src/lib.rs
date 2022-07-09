use error::InternalConnectError;
use hyper::client::connect::HttpConnector;
use hyper::{client::ResponseFuture, Body, Client, Request, Response, Uri};
use hyper_openssl::HttpsConnector;
use openssl::{
    ssl::{SslConnector, SslMethod},
    x509::X509,
};
use std::path::{Path, PathBuf};
use std::{error::Error, task::Poll};
use tonic::body::BoxBody;
use tonic_openssl::ALPN_H2_WIRE;
use tower::Service;

pub mod rpc {
    tonic::include_proto!("lnrpc");
}

/// [`tonic::Status`] is re-exported as `Error` for convenience.
pub type LndClientError = tonic::Status;

// /// This is a convenience type which you most likely want to use instead of raw client.
pub type LndClient = rpc::lightning_client::LightningClient<
    tonic::codegen::InterceptedService<MyChannel, MacaroonInterceptor>,
>;

mod error;

/// Supplies requests with macaroon
#[derive(Clone)]
pub struct MacaroonInterceptor {
    macaroon: String,
}

impl tonic::service::Interceptor for MacaroonInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, LndClientError> {
        request.metadata_mut().insert(
            "macaroon",
            #[allow(deprecated)]
            tonic::metadata::MetadataValue::from_str(&self.macaroon)
                .expect("hex produced non-ascii"),
        );
        Ok(request)
    }
}

async fn load_macaroon(
    path: impl AsRef<Path> + Into<PathBuf>,
) -> Result<String, InternalConnectError> {
    let macaroon =
        tokio::fs::read(&path)
            .await
            .map_err(|error| InternalConnectError::ReadFile {
                file: path.into(),
                error,
            })?;
    Ok(hex::encode(&macaroon))
}

pub async fn get_client(
    lnd_host: String,
    lnd_port: u32,
    lnd_tls_cert_path: String,
    lnd_macaroon_path: String,
) -> Result<LndClient, Box<dyn std::error::Error>> {
    let lnd_address = format!("https://{}:{}", lnd_host, lnd_port).to_string();

    let pem = tokio::fs::read(lnd_tls_cert_path).await.ok();
    let uri = lnd_address.parse::<Uri>().unwrap();
    let channel = MyChannel::new(pem, uri).await?;

    // TODO: don't use unwrap.
    let macaroon = load_macaroon(lnd_macaroon_path).await.unwrap();
    let interceptor = MacaroonInterceptor { macaroon };

    let client = rpc::lightning_client::LightningClient::with_interceptor(channel, interceptor);

    Ok(client)
}

#[derive(Clone)]
pub struct MyChannel {
    uri: Uri,
    client: MyClient,
}

#[derive(Clone)]
enum MyClient {
    ClearText(Client<HttpConnector, BoxBody>),
    Tls(Client<HttpsConnector<HttpConnector>, BoxBody>),
}

impl MyChannel {
    pub async fn new(certificate: Option<Vec<u8>>, uri: Uri) -> Result<Self, Box<dyn Error>> {
        let mut http = HttpConnector::new();
        http.enforce_http(false);
        let client = match certificate {
            None => MyClient::ClearText(Client::builder().http2_only(true).build(http)),
            Some(pem) => {
                let ca = X509::from_pem(&pem[..])?;
                let mut connector = SslConnector::builder(SslMethod::tls())?;
                connector.cert_store_mut().add_cert(ca)?;
                connector.set_alpn_protos(ALPN_H2_WIRE)?;
                let mut https = HttpsConnector::with_connector(http, connector)?;
                https.set_callback(|c, _| {
                    c.set_verify_hostname(false);
                    Ok(())
                });
                MyClient::Tls(Client::builder().http2_only(true).build(https))
            }
        };

        Ok(Self { client, uri })
    }
}

impl Service<Request<BoxBody>> for MyChannel {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = ResponseFuture;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, mut req: Request<BoxBody>) -> Self::Future {
        let uri = Uri::builder()
            .scheme(self.uri.scheme().unwrap().clone())
            .authority(self.uri.authority().unwrap().clone())
            .path_and_query(req.uri().path_and_query().unwrap().clone())
            .build()
            .unwrap();
        *req.uri_mut() = uri;
        match &self.client {
            MyClient::ClearText(client) => client.request(req),
            MyClient::Tls(client) => client.request(req),
        }
    }
}
