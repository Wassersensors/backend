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

    pub type Db = Arc<Mutex<(HashMap<String, String>, HashMap<String, VecDeque<Record>>)>>;

    pub fn blank_db() -> Db {
        Arc::new(Mutex::new((HashMap::new(), HashMap::new())))
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct InputRecord {
        pub rate: f32,
        pub total_volume: f32,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct Record {
        pub timestamp: u128,
        pub rate: f32,
        pub total_volume: f32,
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct OutputRecord {
        pub device_id: String,
        pub alias: String,
        pub record: Record, 
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct RegisterRequest {
        pub alias: String,
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

    pub fn routes(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        record_get(db.clone())
            .or(record_create(db.clone()))
            .or(register_device(db))
    }

    pub fn record_get(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("poll")
            .and(warp::get())
            .and(with_db(db))
            .and_then(handlers::get_record)
    }

    pub fn register_device(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
      warp::path!("register" / String)
          .and(warp::post())
          .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
          .and(with_db(db))
          .and_then(handlers::register_device)
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

    use super::data::{Db, Record, OutputRecord, RegisterRequest, InputRecord};

    pub async fn write_record(device_id: String, input: InputRecord, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut mutex = db.lock().await;
        let records = &mut mutex.1;
        let record = Record {
            timestamp: now(),
            rate: input.rate,
            total_volume: input.total_volume,
        };
        if !records.contains_key(&device_id) {
          records.insert(device_id.clone(), VecDeque::with_capacity(86400));
        }
        let vec = records.get_mut(&device_id).unwrap();
        if vec.len() >= 86400 {
          vec.pop_front();
        }
        vec.push_back(record.clone());

        Ok(warp::reply::json(&record))
    }

    pub async fn register_device(device_id: String, input: RegisterRequest, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut mutex = db.lock().await;
        let devices = &mut mutex.0;
        devices.insert(device_id.clone(), input.alias);

        Ok(warp::http::StatusCode::OK)
    }

    pub async fn get_record(db: Db) -> Result<impl warp::Reply, Infallible> {
        let mutex = db.lock().await;
        let (devices, records) = mutex.clone();
        let default_record = Record::default(); 
        let mut out: Vec<OutputRecord> = Vec::new();
        for key in records.keys() {
          let output_record = OutputRecord {
            device_id: key.clone(),
            alias: devices.get(key).unwrap_or(key).clone(),
            record: records.get(key).unwrap().back().unwrap_or(&default_record).clone()
          };
          out.push(output_record);
        }

        Ok(warp::reply::json(&out))
    }
}