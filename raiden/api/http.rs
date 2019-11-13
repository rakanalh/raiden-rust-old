extern crate futures;
extern crate hyper;
extern crate serde_json;

use futures::{future, Future, Stream};
use hyper::client::HttpConnector;
use hyper::service::service_fn;
use hyper::{header, Body, Client, Method, Request, Response, Server, StatusCode};
use slog::Logger;

static NOTFOUND: &[u8] = b"Not Found";

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;

fn api_post_response(req: Request<Body>) -> ResponseFuture {
    // A web api to run against
    Box::new(
        req.into_body()
            .concat2() // Concatenate all chunks in the body
            .from_err()
            .and_then(|entire_body| {
                // TODO: Replace all unwraps with proper error handling
                let str = String::from_utf8(entire_body.to_vec())?;
                let mut data: serde_json::Value = serde_json::from_str(&str)?;
                data["test"] = serde_json::Value::from("test_value");
                let json = serde_json::to_string(&data)?;
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json))?;
                Ok(response)
            }),
    )
}

fn api_get_response() -> ResponseFuture {
    let data = vec!["foo", "bar"];
    let res = match serde_json::to_string(&data) {
        Ok(json) => Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal Server Error"))
            .unwrap(),
    };

    Box::new(future::ok(res))
}

fn info_endpoint() -> ResponseFuture {
    let data = vec!["foo", "bar"];
    let res = match serde_json::to_string(&data) {
        Ok(json) => Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal Server Error"))
            .unwrap(),
    };

    Box::new(future::ok(res))
}


fn handle_request(req: Request<Body>, _client: &Client<HttpConnector>) -> ResponseFuture {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/info") => info_endpoint(),
        (&Method::POST, "/channels") => api_post_response(req),
        (&Method::GET, "/channels") => api_get_response(),
        _ => {
            // Return 404 not found response.
            let body = Body::from(NOTFOUND);
            Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body)
                    .unwrap(),
            ))
        }
    }
}

pub fn server(log: Logger) -> impl Future<Item = (), Error = ()> {
    let addr = "127.0.0.1:1337".parse().unwrap();

    // Share a `Client` with all `Service`s
    let client = Client::new();

    let new_service = move || {
        // Move a clone of `client` into the `service_fn`.
        let client = client.clone();
        service_fn(move |req| handle_request(req, &client))
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    info!(log, "Listening on http://{}", addr);

    server
}
