use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

// Define a type that can represent either a nested struct or final key-value pairs
#[derive(Debug, Serialize)]
pub enum ConfigValue {
    Struct(HashMap<String, ConfigValue>),
    Values(HashMap<String, String>),
}

pub fn parse_config(input: &str) -> serde_json::Value {
    let lines = input.lines().collect::<Vec<&str>>();
    let mut structs: Vec<String> = Vec::new();
    let mut config_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    let assignment_re = Regex::new(r"^\w+\s*=").unwrap();

    for line in lines.iter() {
        if line.trim().matches("struct.begin").count() > 0 {
            let struct_name = line.split(":").nth(0).unwrap().trim();
            structs.push(struct_name.to_string());
            config_map
                .entry(struct_name.to_string())
                .or_insert(HashMap::new());
        } else if line.trim().matches("struct.end").count() > 0 {
            structs.pop().expect("Failed to pop. This appears to be an error in parsing the beginning or ending of a struct.");
        } else if assignment_re.is_match(line.trim()) {
            let name = line.trim().split("=").nth(0).unwrap().trim();
            let value = line.trim().split_once('=').unwrap().1.trim();

            if let Some(current_struct) = structs.last() {
                config_map
                    .entry(current_struct.clone())
                    .or_insert(HashMap::new())
                    .insert(name.to_string(), value.to_string());
            }
        }
    }

    serde_json::to_value(&config_map).expect("Failed to convert to JSON value")
}
