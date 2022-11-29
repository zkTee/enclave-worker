use std::sync::Mutex;
use std::collections::HashMap;

lazy_static! {
    pub static ref DATABASE: Mutex<HashMap<Vec<u8>, Vec<u8>>> = {
        let db = HashMap::new();
        Mutex::new(db)
    };
}
