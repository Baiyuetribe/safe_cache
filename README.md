# SAFE_Cache

## Features

- [x] k-v 键值缓存,v 值为泛型，支持任意类型
- [x] Mutex 并发安全
- [x] 简单有效

## Install

```bash
cargo add safe_cache
```

## Usage

场景 1：定义缓存及清理时间

```rust
use safe_cache::{Cache, async_cleanup_task};
use std::sync::Arc;

async fn main() {
    let cache = Arc::new(Cache::new());

    // 启动定时任务清理内存
    let cache_clone = Arc::clone(&cache);
    async_cleanup_task(cache_clone, 10).await; // 10秒清理一次


    // 示例使用
     // 示例使用
    cache.set("number".to_string(), 42, 20);
    cache.set("list".to_string(), vec![1, 2, 3], 60);
    cache.set("text".to_string(), "Hello, Rust!".to_string(), 0); // 0代表永不过期

    println!("Value for number: {:?}", cache.get::<u16>("number"));
    println!("Value for list: {:?}", cache.get::<Vec<i32>>("list"));
    println!("Value for text: {:?}", cache.get::<Vec<i32>>("text"));

    // 休眠等待定时任务执行
    thread::sleep(Duration::from_secs(30));

    println!("After clearing expired entries:");
    println!("Value for number: {:?}", cache.get::<i32>("number"));
    println!("Value for text: {:?}", cache.get::<String>("text"));
    cache.remove("text");
    println!("Value for text: {:?}", cache.get::<String>("text"));
}
```

场景 2：当做全局 config 来使用，不启用清理任务

```rust
use safe_cache::Cache;
use std::sync::Arc;

fn main() {
    let cache = Arc::new(Cache::new());

    // 示例使用
     // 示例使用
    cache.set("number".to_string(), 42, 0);
    cache.set("list".to_string(), vec![1, 2, 3], 0);
    cache.set("text".to_string(), "Hello, Rust!".to_string(), 0); // 0代表永不过期

    println!("Value for number: {:?}", cache.get::<i32>("number"));
    println!("Value for list: {:?}", cache.get::<Vec<i32>>("list"));
    println!("Value for text: {:?}", cache.get::<Vec<String>>("text"));
}
```

## Overview

```rust
impl Cache<T> {
    fn set<T: 'static + Clone + Send>(&self, key: String, value: T, expire_seconds: u64);
    fn get<T: 'static + Clone>(&self, key: &str) -> Option<T>;
    fn remove(&self, key: &str);
}
```
