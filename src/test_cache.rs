use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

static CACHE: LazyLock<Arc<Mutex<HashMap<String, String>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

#[tokio::main]
async fn main() {
    {
        let mut cache = CACHE.clone().lock_owned().await;
        cache.insert("key".to_string(), "value".to_string());
    }

    {
        let cache = CACHE.lock().await;
        println!("Value: {:?}", cache.get("key"));
        assert_eq!(cache.get("key"), Some(&"value".to_string()));
    }
}
