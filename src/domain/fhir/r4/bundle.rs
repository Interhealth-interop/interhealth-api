use serde_json::{json, Value};

pub fn get_template() -> Value {
    json!({
        "resourceType": "Bundle",
        "type": "transaction",
        "entry": []
    })
}