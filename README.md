# SAFE_Cache

## Features

- [x] k-v 键值缓存,v 值为泛型，支持任意类型
- [x] 并发安全
- [x] 简单有效

## Install

```bash
cargo add safe_cache
```

## Usage

场景 1：定义缓存及清理时间

```rust
use safe_cache::{Cache, start_cleanup_thread};

fn main() {
    let cache = Arc::new(Cache::new());

    // 启动定时任务清理内存
    let cache_clone = Arc::clone(&cache);
    start_cleanup_thread(cache_clone, 10); // 10秒清理一次


    // 示例使用
     // 示例使用
    cache.insert("number".to_string(), 42, 20);
    cache.insert("list".to_string(), vec![1, 2, 3], 60);
    cache.insert("text".to_string(), "Hello, Rust!".to_string(), 0); // 0代表永不过期

    println!("Value for number: {:?}", cache.get::<u16>("number"));
    println!("Value for list: {:?}", cache.get::<Vec<i32>>("list"));
    println!("Value for text: {:?}", cache.get::<Vec<i32>>("text"));

    // 休眠等待定时任务执行
    thread::sleep(Duration::from_secs(30));

    println!("After clearing expired entries:");
    println!("Value for number: {:?}", cache.get("number"));
    println!("Value for text: {:?}", cache.get("text"));
    cache.remove("text");
    println!("Value for text: {:?}", cache.get("text"));
}
```

场景 2：当做全局 config 来使用，不启用清理任务

```rust
use safe_cache::{Cache, start_cleanup_thread};

fn main() {
    let cache = Arc::new(Cache::new());

    // // 启动定时任务清理内存
    // let cache_clone = Arc::clone(&cache);
    // start_cleanup_thread(cache_clone, 10); // 10秒清理一次

    // 示例使用
     // 示例使用
    cache.insert("number".to_string(), 42, 0);
    cache.insert("list".to_string(), vec![1, 2, 3], 0);
    cache.insert("text".to_string(), "Hello, Rust!".to_string(), 0); // 0代表永不过期

    println!("Value for number: {:?}", cache.get::<u16>("number"));
    println!("Value for list: {:?}", cache.get::<Vec<i32>>("list"));
    println!("Value for text: {:?}", cache.get::<Vec<i32>>("text"));
}
```

场景 3：子函数传递

```rust
use safe_cache::{Cache, start_cleanup_thread};

// 示例函数2
fn sub_fun(cache: Arc<Cache>) {
    cache.insert("sub_key1".to_string(), "sub_key1_vulue".to_string(), 0); // 永不过期
}

fn main() {
    let cache = Arc::new(Cache::new());

    // 启动定时任务清理内存
    let cache_clone = Arc::clone(&cache);
    start_cleanup_thread(cache_clone);

    // 示例使用
    cache.insert("key1".to_string(), "value1".to_string(), 20);
    cache.insert("key2".to_string(), "value2".to_string(), 0); // 永不过期

    println!("Value for key1: {:?}", cache.get("key1"));
    println!("Value for key2: {:?}", cache.get("key2"));
    sub_fun(cache.clone());
    println!("Value for text: {:?}", cache.get::<Vec<String>>("sub_key1"));
    cache.remove("sub_key1");
    println!("Value for text: {:?}", cache.get::<Vec<String>>("sub_key1"));
}
```
