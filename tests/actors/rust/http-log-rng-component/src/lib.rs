wit_bindgen::generate!("actor");

use serde::Deserialize;
use serde_json::json;
use wasmbus_rpc::common::{deserialize, serialize};
use wasmcloud_actor::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse};

struct Actor;

impl guest::Guest for Actor {
    fn call(operation: String, payload: Option<Vec<u8>>) -> Result<Option<Vec<u8>>, String> {
        assert_eq!(operation, "HttpServer.HandleRequest");
        let payload = payload.expect("missing payload");
        let HttpRequest {
            method,
            path,
            query_string,
            header,
            body,
        } = deserialize(payload.as_ref()).expect("failed to deserialize request");
        assert!(method.is_empty());
        assert!(path.is_empty());
        assert!(query_string.is_empty());
        assert!(header.is_empty());

        #[derive(Deserialize)]
        struct Request {
            min: u32,
            max: u32,
        }
        let Request { min, max } =
            serde_json::from_slice(&body).expect("failed to decode request body");

        logging::log(logging::Level::Trace, "trace-context", "trace");
        logging::log(logging::Level::Debug, "debug-context", "debug");
        logging::log(logging::Level::Info, "info-context", "info");
        logging::log(logging::Level::Warn, "warn-context", "warn");
        logging::log(logging::Level::Error, "error-context", "error");

        let res = json!({
            "guid": HostRng::generate_guid().to_string(),
            "random_in_range": HostRng::random_in_range(min, max),
            "random_32": HostRng::random32(),
        });
        eprintln!("response: `{res:?}`");
        let body = serde_json::to_vec(&res).expect("failed to encode response to JSON");

        let res = serialize(&HttpResponse {
            body,
            ..Default::default()
        })
        .expect("failed to serialize response");
        Ok(Some(res))
    }
}

export_actor!(Actor);
