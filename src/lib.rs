use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

pub struct Cache {
    data: Mutex<HashMap<String, (Arc<Mutex<dyn Any + Send>>, Option<SystemTime>)>>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            data: Mutex::new(HashMap::new()),
        }
    }

    pub fn get<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        let data = self.data.lock().unwrap();
        if let Some((value, _)) = data.get(key) {
            let value = value.lock().unwrap();
            value.downcast_ref::<T>().cloned()
        } else {
            None
        }
    }
    pub fn set<T: 'static + Clone + Send>(&self, key: String, value: T, expire_seconds: u64) {
        let expiration_time = if expire_seconds == 0 {
            None
        } else {
            Some(SystemTime::now() + Duration::from_secs(expire_seconds))
        };
        let entry = (
            Arc::new(Mutex::new(value)) as Arc<Mutex<dyn Any + Send>>,
            expiration_time,
        );

        let mut data = self.data.lock().unwrap();
        if data.len() > 10240 {
            // set max cache size
            data.clear();
        }
        data.insert(key, entry);
    }

    pub fn remove(&self, key: &str) {
        let mut data = self.data.lock().unwrap();
        data.remove(key);
    }

    pub fn clear_expired_entries(&self) {
        let mut data = self.data.lock().unwrap();
        let now = SystemTime::now();
        data.retain(|_, (_, expiration_time)| expiration_time.map_or(true, |et| et > now));
    }
}

// block_cleanup_task 会阻塞当前线程，所以一般不会用这个函数
// pub fn block_cleanup_task(cache: Arc<Cache>, secs: u64) {
//     std::thread::spawn(move || {
//         loop {
//             std::thread::sleep(Duration::from_secs(secs)); // 每几秒清理一次
//             cache.clear_expired_entries();
//         }
//     });
// }

pub async fn async_cleanup_task(cache: Arc<Cache>, secs: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(secs));
        loop {
            interval.tick().await;
            cache.clear_expired_entries();
        }
    });
}

#[cfg(test)]
async fn test_cache() {
    let cache = Arc::new(Cache::new());
    let cache1 = cache.clone();
    async_cleanup_task(cache1, 1).await;
    cache.set("a".to_string(), 1, 1);
    assert_eq!(cache.get::<i32>("a"), Some(1));
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_eq!(cache.get::<i32>("a"), None);
}
