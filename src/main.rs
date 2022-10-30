use backend::{data, filters};

#[tokio::main]
async fn main() {
    let db = data::blank_db();
    let api = filters::records(db);

    warp::serve(api)
        .run(([127, 0, 0, 1], 42069))
        .await;
}
