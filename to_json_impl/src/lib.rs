use serde_json::{from_str, Value};

pub trait ToJson {
    fn to_json_string(&self) -> String;
    fn to_json(&self) -> Value {
        from_str(&self.to_json_string()).expect("Failed to deserialize from JSON")
    }
    fn get_string(&self) -> String {
        self.to_json_string().replace("\"", "")
    }
}
