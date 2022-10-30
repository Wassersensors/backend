pub mod util {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn now() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}

pub mod data {
    use serde::{Deserialize, Serialize};
    use std::{sync::Arc, collections::VecDeque};
    use tokio::sync::Mutex;

    use crate::util::now;

    pub type Db = Arc<Mutex<VecDeque<Record>>>;

    pub fn blank_db() -> Db {
        Arc::new(Mutex::new(VecDeque::with_capacity(86400)))
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct InputRecord {
        pub rate: f32,
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct Record {
        pub timestamp: u128,
        pub rate: f32,
    }

    impl Record {
        pub fn default() -> Record {
            Record {
                timestamp: now(),
                rate: 0.0,
            }
        }
    }
}

pub mod filters {
    use warp::Filter;

    use crate::{data::Db, handlers};

    pub fn records(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        record_get(db.clone())
            .or(record_create(db))
    }

    pub fn record_get(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("poll")
            .and(warp::get())
            .and(with_db(db))
            .and_then(handlers::get_record)
    }

    pub fn record_create(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("record")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
        .and(with_db(db))
        .and_then(handlers::write_record)
    }

    fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}

pub mod handlers {
    use std::convert::Infallible;

    use crate::util::now;

    use super::data::{Db, Record, InputRecord};

    pub async fn write_record(input: InputRecord, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut vec = db.lock().await;
        let record = Record {
            timestamp: now(),
            rate: input.rate,
        };
        if vec.len() >= 86400 {
            _ = vec.pop_front();
        }
        vec.push_back(record.clone());

        Ok(warp::reply::json(&record))
    }

    pub async fn get_record(db: Db) -> Result<impl warp::Reply, Infallible> {
        let records = db.lock().await;
        let records = records.clone();
        let default_record = Record::default();
        let record = records.back().unwrap_or(&default_record);

        Ok(warp::reply::json(&record))
    }
}