use backend::{data, filters};
use warp::{Filter, hyper::HeaderMap, http::HeaderValue};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));

    let db = data::blank_db();
    let api = filters::routes(db);

    let log = warp::log("backend");

    let routes = api.with(log);
    let routes = routes.with(warp::reply::with::headers(headers));

    warp::serve(routes)
        .run(([0, 0, 0, 0], 42069))
        .await;
}
