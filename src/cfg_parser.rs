use regex::Regex;
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::collections::HashMap;

// Define a type that can represent either a nested struct or final key-value pairs
#[derive(Debug)]
pub enum ConfigValue {
    Struct(HashMap<String, ConfigValue>),
    Value(String),
}

// Add custom serialization
impl Serialize for ConfigValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ConfigValue::Value(s) => serializer.serialize_str(s),
            ConfigValue::Struct(map) => {
                let mut map_ser = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    map_ser.serialize_entry(k, v)?;
                }
                map_ser.end()
            }
        }
    }
}

pub fn parse_config(input: &str) -> serde_json::Value {
    let lines = input.lines().collect::<Vec<&str>>();
    let mut structs: Vec<String> = Vec::new();
    let mut config: ConfigValue = ConfigValue::Struct(HashMap::new());
    let assignment_re = Regex::new(r"^\w+\s*=").unwrap();

    for line in lines.iter() {
        if line.trim().matches("struct.begin").count() > 0 {
            let struct_name = line.split(":").nth(0).unwrap().trim();
            structs.push(struct_name.to_string());

            // Create nested structure
            let mut current = &mut config;
            for struct_name in &structs {
                current = match current {
                    ConfigValue::Struct(map) => map
                        .entry(struct_name.clone())
                        .or_insert_with(|| ConfigValue::Struct(HashMap::new())),
                    _ => panic!("Expected a struct"),
                };
            }
        } else if line.trim().matches("struct.end").count() > 0 {
            structs.pop().expect("Failed to pop. This appears to be an error in parsing the beginning or ending of a struct.");
        } else if assignment_re.is_match(line.trim()) {
            let name = line.trim().split("=").nth(0).unwrap().trim();
            let value = line.trim().split_once('=').unwrap().1.trim();

            // Insert value into current struct
            let mut current = &mut config;
            for struct_name in &structs {
                current = match current {
                    ConfigValue::Struct(map) => map
                        .entry(struct_name.clone())
                        .or_insert_with(|| ConfigValue::Struct(HashMap::new())),
                    _ => panic!("Expected a struct"),
                };
            }

            match current {
                ConfigValue::Struct(map) => {
                    map.insert(name.to_string(), ConfigValue::Value(value.to_string()));
                }
                _ => panic!("Expected a struct"),
            }
        }
    }

    serde_json::to_value(&config).expect("Failed to convert to JSON value")
}

pub fn json_to_cfg(value: &serde_json::Value) -> String {
    fn write_value(
        builder: &mut String,
        value: &serde_json::Value,
        name: &str,
        indent_level: usize,
    ) {
        let indent = "   ".repeat(indent_level);

        match value {
            serde_json::Value::Object(map) => {
                // Write struct begin
                builder.push_str(&format!("{}{} : struct.begin\n", indent, name));

                // Write all key-value pairs inside the struct
                for (key, val) in map {
                    write_value(builder, val, key, indent_level + 1);
                }

                // Write struct end
                builder.push_str(&format!("{}struct.end\n", indent));
            }
            _ => {
                // Write simple key-value pair
                builder.push_str(&format!(
                    "{}{} = {}\n",
                    indent,
                    name,
                    value.as_str().unwrap_or(&value.to_string())
                ));
            }
        }
    }

    let mut output = String::new();
    if let serde_json::Value::Object(map) = value {
        for (key, val) in map {
            write_value(&mut output, val, key, 0);
        }
    }

    output
}
