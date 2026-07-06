use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

pub struct PasscodeStore {
    passcodes: Arc<Mutex<HashMap<String, (String, chrono::DateTime<chrono::Utc>)>>>,
}

impl PasscodeStore {
    pub fn new() -> Self {
        Self {
            passcodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn generate_passcode(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..1_000_000))
    }

    pub async fn store_passcode(&self, device_id: String, passcode: String) {
        let expires_at = Utc::now() + Duration::minutes(5);
        let mut store = self.passcodes.lock().await;
        store.insert(device_id, (passcode, expires_at));
        store.retain(|_, (_, exp)| *exp > Utc::now());
    }

    pub async fn verify_passcode(&self, device_id: &str, passcode: &str) -> bool {
        let store = self.passcodes.lock().await;
        if let Some((stored_passcode, expires_at)) = store.get(device_id) {
            if expires_at > &Utc::now() && stored_passcode == passcode {
                return true;
            }
        }
        false
    }

    pub async fn remove_passcode(&self, device_id: &str) {
        let mut store = self.passcodes.lock().await;
        store.remove(device_id);
    }
}

static PASSCODE_STORE: OnceLock<PasscodeStore> = OnceLock::new();

pub fn get_passcode_store() -> &'static PasscodeStore {
    PASSCODE_STORE.get_or_init(PasscodeStore::new)
}
