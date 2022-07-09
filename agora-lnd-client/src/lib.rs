use error::InternalConnectError;
use hyper::{client::connect::HttpConnector, Client, Uri};
use hyper_openssl::HttpsConnector;
use openssl::{
    ssl::{SslConnector, SslMethod},
    x509::X509,
};
use std::path::{Path, PathBuf};
use tonic_openssl::ALPN_H2_WIRE;
use tower::util::ServiceFn;

pub mod rpc {
    tonic::include_proto!("lnrpc");
}

/// [`tonic::Status`] is re-exported as `Error` for convenience.
pub type Error = tonic::Status;

mod error;

/// This is a convenience type which you most likely want to use instead of raw client.
pub type LndClient = rpc::lightning_client::LightningClient<
    tonic::codegen::InterceptedService<
        ServiceFn<hyper::Request<tonic::body::BoxBody>>,
        MacaroonInterceptor,
    >,
>;

/// Supplies requests with macaroon
#[derive(Clone)]
pub struct MacaroonInterceptor {
    macaroon: String,
}

impl tonic::service::Interceptor for MacaroonInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Error> {
        request.metadata_mut().insert(
            "macaroon",
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

#[tokio::main]
async fn main() -> Result<LndClient, Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let pem = tokio::fs::read("example/tls/ca.pem").await?;
    let ca = X509::from_pem(&pem[..])?;
    let mut connector = SslConnector::builder(SslMethod::tls())?;
    connector.cert_store_mut().add_cert(ca)?;
    connector.set_alpn_protos(ALPN_H2_WIRE)?;

    let mut http = HttpConnector::new();
    http.enforce_http(false);
    let mut https = HttpsConnector::with_connector(http, connector)?;

    // This is set because we are currently sending
    // `[::1]:50051` as the hostname but the cert was
    // originally signed with `example.com`. This will
    // disable hostname checking and it is BAD! DON'T DO IT!
    https.set_callback(|c, _| {
        c.set_verify_hostname(false);
        Ok(())
    });

    // Configure hyper's client to be h2 only and build with the
    // correct https connector.
    let hyper = Client::builder().http2_only(true).build(https);

    let uri = Uri::from_static("https://[::1]:50051");

    // Hyper's client requires that requests contain full Uris include a scheme and
    // an authority. Tonic's transport will handle this for you but when using the client
    // manually you need ensure the uri's are set correctly.
    let add_origin = tower::service_fn(|mut req: hyper::Request<tonic::body::BoxBody>| {
        let uri = Uri::builder()
            .scheme(uri.scheme().unwrap().clone())
            .authority(uri.authority().unwrap().clone())
            .path_and_query(req.uri().path_and_query().unwrap().clone())
            .build()
            .unwrap();

        *req.uri_mut() = uri;

        hyper.request(req)
    });

    // let client = rpc::lightning_client::LightningClient::new(add_origin);

    let macaroon = load_macaroon("macaroon_file.macaroon").await?;

    let interceptor = MacaroonInterceptor { macaroon };

    // Ok(client)
    Ok(rpc::lightning_client::LightningClient::with_interceptor(
        add_origin,
        interceptor,
    ))
}
