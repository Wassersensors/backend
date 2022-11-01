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
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    use crate::util::now;

    pub type Db = Arc<Mutex<HashMap<String, VecDeque<Record>>>>;

    pub fn blank_db() -> Db {
        Arc::new(Mutex::new(HashMap::new()))
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct InputRecord {
        pub rate: f32,
        pub total_volume: f32,
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct Record {
        pub timestamp: u128,
        pub rate: f32,
        pub total_volume: f32,
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct OutputRecord {
        pub device_id: String,
        pub record: Record, 
    }

    impl Record {
        pub fn default() -> Record {
            Record {
                timestamp: now(),
                rate: 0.0,
                total_volume: 0.0,
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
        warp::path!("record" / String)
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
    use std::collections::VecDeque;
    use crate::util::now;

    use super::data::{Db, Record, OutputRecord, InputRecord};

    pub async fn write_record(device_id: String, input: InputRecord, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut map = db.lock().await;
        let record = Record {
            timestamp: now(),
            rate: input.rate,
            total_volume: input.total_volume,
        };
        if !map.contains_key(&device_id) {
          map.insert(device_id.clone(), VecDeque::with_capacity(86400));
        }
        let vec = map.get_mut(&device_id).unwrap();
        if vec.len() >= 86400 {
          vec.pop_front();
        }
        vec.push_back(record.clone());

        Ok(warp::reply::json(&record))
    }

    pub async fn get_record(db: Db) -> Result<impl warp::Reply, Infallible> {
        let records_map = db.lock().await;
        let records_map = records_map.clone();
        let default_record = Record::default(); 
        let mut out: Vec<OutputRecord> = Vec::new();
        for key in records_map.keys() {
          let output_record = OutputRecord {
            device_id: key.clone(),
            record: records_map.get(key).unwrap().back().unwrap_or(&default_record).clone()
          };
          out.push(output_record);
        }

        Ok(warp::reply::json(&out))
    }
}