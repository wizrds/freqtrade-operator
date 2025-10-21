use serde_json::Value;


pub fn check_key_exists(payload: &Value, key: &str) -> bool {
    let mut current_value = payload;

    for part in key.split('.') {
        if let Some(next_value) = current_value.get(part) {
            current_value = next_value;
        } else {
            return false;
        }
    }
    true
}