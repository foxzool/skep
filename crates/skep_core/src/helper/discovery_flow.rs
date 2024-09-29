use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscoveryKey {
    pub domain: String,
    pub key: Key,
    pub version: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Key {
    Single(String),
    Multiple(Vec<String>),
}

impl DiscoveryKey {
    fn from_json_dict(json_dict: &serde_json::Value) -> Self {
        let key = match &json_dict["key"] {
            serde_json::Value::Array(arr) => Key::Multiple(
                arr.iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect(),
            ),
            serde_json::Value::String(s) => Key::Single(s.clone()),
            _ => panic!("Invalid key type"),
        };
        DiscoveryKey {
            domain: json_dict["domain"].as_str().unwrap().to_string(),
            key,
            version: json_dict["version"].as_i64().unwrap() as i32,
        }
    }
}
