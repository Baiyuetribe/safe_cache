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

pub fn start_cleanup_thread(cache: Arc<Cache>, secs: u64) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(secs)); // 每几秒清理一次
            cache.clear_expired_entries();
        }
    });
}

fn main() -> () {
    let cache = Arc::new(Cache::new());

    // 启动定时任务清理内存
    let cache_clone = Arc::clone(&cache);
    start_cleanup_thread(cache_clone, 10);

    // 示例使用
    cache.set("number".to_string(), 42, 60);
    cache.set("list".to_string(), vec![1, 2, 3], 60);
    cache.set("text".to_string(), "Hello, Rust!".to_string(), 120);

    println!("Value for number: {:?}", cache.get::<u16>("number"));
    println!("Value for number: {:?}", cache.get::<i32>("number"));
    println!("Value for list: {:?}", cache.get::<Vec<i32>>("list"));
    cache.remove("list");
    println!("Value for list: {:?}", cache.get::<Vec<i32>>("number2"));
    println!("Value for text: {:?}", cache.get::<String>("text"));
    println!("Value for text: {:?}", cache.get::<String>("text2"));
    let a = cache
        .get::<String>("text2")
        .unwrap_or_else(|| "default".to_string());
    println!("{:?}", a);

    // 休眠等待定时任务执行
    std::thread::sleep(Duration::from_secs(70));

    println!("After clearing expired entries:");
    println!("Value for number: {:?}", cache.get::<i32>("number"));
    println!("Value for text: {:?}", cache.get::<String>("text"));
}
