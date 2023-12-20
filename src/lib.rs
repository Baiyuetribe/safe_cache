use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
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
        let data = self.data.lock();
        let data = match data {
            Err(_) => return None,
            Ok(v) => v,
        };
        if let Some((value, _)) = data.get(key) {
            let value = value.lock();
            match value {
                Err(_) => return None,
                Ok(value) => return value.downcast_ref::<T>().cloned(),
            }
        }
        None
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

        let data = self.data.lock();
        let mut data = match data {
            Err(_) => return,
            Ok(v) => v,
        };
        if data.len() > 10240 {
            // set max cache size
            data.clear();
        }
        data.insert(key, entry);
    }

    pub fn remove(&self, key: &str) {
        let data = self.data.lock();
        match data {
            Err(_) => return,
            Ok(mut v) => {
                v.remove(key);
            }
        }
    }

    pub fn clear_expired_entries(&self) {
        let data = self.data.lock();
        let mut data = match data {
            Err(_) => return,
            Ok(v) => v,
        };
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

pub struct CacheRwLock {
    data: RwLock<HashMap<String, (Arc<RwLock<Box<dyn Any + Send + Sync>>>, Option<SystemTime>)>>,
}

impl CacheRwLock {
    pub fn new() -> Self {
        CacheRwLock {
            data: RwLock::new(HashMap::new()),
        }
    }

    pub fn get<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        let data = self.data.read();
        let data = match data {
            Err(_) => return None,
            Ok(v) => v,
        };
        if let Some((value, _)) = data.get(key) {
            let value = value.read();
            match value {
                Err(_) => return None,
                Ok(v) => return v.downcast_ref::<T>().cloned(),
            };
        }
        None
    }

    pub fn set<T: 'static + Clone + Send + Sync>(
        &self,
        key: String,
        value: T,
        expire_seconds: u64,
    ) {
        let expiration_time = if expire_seconds == 0 {
            None
        } else {
            Some(SystemTime::now() + Duration::from_secs(expire_seconds))
        };
        let entry = (
            Arc::new(RwLock::new(Box::new(value) as Box<dyn Any + Send + Sync>)),
            expiration_time,
        );

        let data = self.data.write();
        let mut data = match data {
            Err(_) => return,
            Ok(v) => v,
        };
        if data.len() > 10240 {
            // set max cache size
            data.clear();
        }
        data.insert(key, entry);
    }

    pub fn remove(&self, key: &str) {
        let data = self.data.write();
        let mut data = match data {
            Err(_) => return,
            Ok(v) => v,
        };
        data.remove(key);
    }

    pub fn clear_expired_entries(&self) {
        let data = self.data.write();
        let mut data = match data {
            Err(_) => return,
            Ok(v) => v,
        };
        let now = SystemTime::now();
        data.retain(|_, &mut (_, expiration_time)| match expiration_time {
            Some(time) => now <= time,
            None => true,
        });
    }
}

pub async fn async_cleanup_task_rwlock(cache: Arc<CacheRwLock>, secs: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(secs));
        loop {
            interval.tick().await;
            cache.clear_expired_entries();
        }
    });
}

#[cfg(test)]
async fn test_cache_rwlock() {
    let cache = Arc::new(CacheRwLock::new());
    let cache1 = cache.clone();
    async_cleanup_task_rwlock(cache1, 1).await;
    cache.set("a".to_string(), 1, 1);
    assert_eq!(cache.get::<i32>("a"), Some(1));
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_eq!(cache.get::<i32>("a"), None);
}
