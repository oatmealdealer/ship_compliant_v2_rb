use hyper::header::HeaderValue;
use magnus::{function, prelude::*, Error, Ruby, TryConvert};
use ship_compliant_v2_rs::prelude::Client;

// Copied directly from reqwest since they don't yet support setting default basic auth in the ClientBuilder
pub fn basic_auth<U, P>(username: U, password: Option<P>) -> HeaderValue
where
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    use base64::prelude::BASE64_STANDARD;
    use base64::write::EncoderWriter;
    use std::io::Write;

    let mut buf = b"Basic ".to_vec();
    {
        let mut encoder = EncoderWriter::new(&mut buf, &BASE64_STANDARD);
        let _ = write!(encoder, "{username}:");
        if let Some(password) = password {
            let _ = write!(encoder, "{password}");
        }
    }
    let mut header = HeaderValue::from_bytes(&buf).expect("base64 is always valid HeaderValue");
    header.set_sensitive(true);
    header
}

#[magnus::wrap(class = "ShipCompliantV2::Client")]
pub struct V2Client {
    inner: Client,
}

impl V2Client {
    pub fn new(baseurl: String, username: String, password: String) -> Result<Self, magnus::Error> {
        let mut headers = reqwest::header::HeaderMap::with_capacity(1);
        headers.insert(
            reqwest::header::AUTHORIZATION,
            basic_auth(username, Some(password)),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("unable to build client");
        Ok(Self {
            inner: Client::new_with_client(&baseurl, client),
        })
    }
    pub fn define_ruby_class(ruby: &Ruby, module: &magnus::RModule) -> Result<(), magnus::Error> {
        let class = module.define_class("Client", ruby.class_object())?;
        class.define_singleton_method("new", function!(V2Client::new, 3))?;
        Ok(())
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("ShipCompliantV2")?;
    V2Client::define_ruby_class(ruby, &module)?;
    Ok(())
}
