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
    let mut config_map: HashMap<String, ConfigValue> = HashMap::new();
    let assignment_re = Regex::new(r"^\w+\s*=").unwrap();

    for line in lines.iter() {
        if line.trim().matches("struct.begin").count() > 0 {
            let struct_name = line.split(":").nth(0).unwrap().trim();
            structs.push(struct_name.to_string());

            // Create nested structure
            let mut current_map = &mut config_map;
            for (i, struct_name) in structs.iter().enumerate() {
                current_map = match current_map
                    .entry(struct_name.clone())
                    .or_insert(ConfigValue::Struct(HashMap::new()))
                {
                    ConfigValue::Struct(map) => map,
                    _ => panic!("Expected a struct"),
                };
            }
            current_map
                .entry(struct_name.to_string())
                .or_insert(ConfigValue::Values(HashMap::new()));
        } else if line.trim().matches("struct.end").count() > 0 {
            structs.pop().expect("Failed to pop. This appears to be an error in parsing the beginning or ending of a struct.");
        } else if assignment_re.is_match(line.trim()) {
            let name = line.trim().split("=").nth(0).unwrap().trim();
            let value = line.trim().split_once('=').unwrap().1.trim();

            // Navigate to the current struct and insert the value
            let mut current_map = &mut config_map;
            for struct_name in &structs {
                current_map = match current_map
                    .entry(struct_name.clone())
                    .or_insert(ConfigValue::Struct(HashMap::new()))
                {
                    ConfigValue::Struct(map) => map,
                    _ => panic!("Expected a struct"),
                };
            }

            if let ConfigValue::Values(values) = current_map
                .entry(structs.last().unwrap().clone())
                .or_insert(ConfigValue::Values(HashMap::new()))
            {
                values.insert(name.to_string(), value.to_string());
            }
        }
    }

    return serde_json::to_value(&config_map).expect("Failed to convert to JSON value");
}
