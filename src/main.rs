use backend::{data, filters};
use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = data::blank_db();
    let api = filters::routes(db);

    let cors = warp::cors().allow_any_origin();
    let log = warp::log("backend");

    let routes = api.with(log);
    let routes = routes.with(cors);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 42069))
        .await;
}
