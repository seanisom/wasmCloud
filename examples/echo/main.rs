use anyhow::{bail, Context};
use wascap::prelude::{ClaimsBuilder, KeyPair};
use wascap::wasm::embed_claims;
use wascap::{caps, jwt};
use wasmbus_rpc::common::{deserialize, serialize};
use wasmcloud_host::{Actor, Runtime};
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let issuer = KeyPair::new_account();
    let module = KeyPair::new_module();
    let claims = ClaimsBuilder::new()
        .issuer(&issuer.public_key())
        .subject(&module.public_key())
        .with_metadata(jwt::Actor {
            name: Some("echo".into()),
            caps: Some(vec![caps::HTTP_SERVER.into()]),
            ..Default::default()
        })
        .build();
    let wasm = embed_claims(test_actors::RUST_ECHO_MODULE, &claims, &issuer)
        .context("failed to embed actor claims")?;

    let rt = Runtime::new().context("failed to construct runtime")?;

    let actor = Actor::read(&rt, wasm.as_slice())
        .await
        .context("failed to construct actor")?;
    let buf = serialize(&HttpRequest::default()).context("failed to encode request")?;
    match actor
        .call("HttpServer.HandleRequest", Some(buf))
        .await
        .context("failed to call `HttpServer.HandleRequest`")?
    {
        Ok(Some(response)) => {
            let HttpResponse {
                status_code,
                header,
                body,
            } = deserialize(&response).context("failed to deserialize response")?;
            println!("Status code: {status_code}");
            for (k, v) in header {
                println!("Header `{k}`: `{v:?}`");
            }
            let body = String::from_utf8(body).context("failed to convert body to UTF-8")?;
            println!("Body: {body}");
            Ok(())
        }
        Ok(None) => bail!("actor did not return a response"),
        Err(err) => bail!("actor failed with: {err}"),
    }
}
