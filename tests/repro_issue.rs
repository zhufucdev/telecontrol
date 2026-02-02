use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

static CACHE: LazyLock<Arc<Mutex<HashMap<String, String>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

#[tokio::test]
async fn test_cache_mechanism() {
    {
        let mut cache = CACHE.lock().await;
        (*cache).insert("key".to_string(), "value".to_string());
    }

    {
        let cache = CACHE.lock().await;
        assert_eq!(cache.get("key"), Some(&"value".to_string()));
    }
}
