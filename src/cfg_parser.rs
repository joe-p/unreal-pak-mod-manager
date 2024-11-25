use crate::merge;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use slotmap::DefaultKey;
use slotmap::SlotMap;
use std::fmt::{Display, Formatter, Result};

#[derive(Serialize, Deserialize, Debug)]
pub struct GscCfgStruct {
    pub name: String,
    pub meta: String,
    pub values: Vec<GscCfgValue>,
    pub parent: Option<DefaultKey>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GscCfgValue {
    pub name: String,
    pub value: Option<String>,
    pub struct_key: Option<DefaultKey>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GscCfg {
    name: String,
    structs: SlotMap<DefaultKey, GscCfgStruct>,
    root_structs: Vec<DefaultKey>,
}

impl Display for GscCfg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for struct_key in self.root_structs.clone() {
            write!(f, "{}", self.struct_to_string(struct_key, 0))?;
        }

        Ok(())
    }
}

impl GscCfg {
    fn struct_to_string(&self, struct_key: DefaultKey, indent: usize) -> String {
        let struct_value = self
            .structs
            .get(struct_key)
            .expect("The given struct key does not exist");

        let mut result = String::new();

        std::fmt::write(
            &mut result,
            format_args!(
                "{}{}\n",
                " ".repeat(indent),
                format!("{} : struct.begin{}", struct_value.name, struct_value.meta)
            ),
        )
        .expect("Error writing to string");

        for cfg_value in &struct_value.values {
            if let Some(value) = &cfg_value.value {
                std::fmt::write(
                    &mut result,
                    format_args!("{}{} = {}\n", " ".repeat(indent + 3), cfg_value.name, value),
                )
                .expect("Error writing to string");
            } else if let Some(nested_key) = cfg_value.struct_key {
                result.push_str(&self.struct_to_string(nested_key, indent + 3));
            }
        }

        std::fmt::write(
            &mut result,
            format_args!("{}{}\n", " ".repeat(indent), "struct.end"),
        )
        .expect("Error writing to string");

        result
    }

    pub fn from_str(name: String, cfg_str: &str) -> GscCfg {
        let mut root_structs: Vec<DefaultKey> = Vec::new();
        let mut structs: SlotMap<DefaultKey, GscCfgStruct> = SlotMap::new();

        let struct_begin_regex: Regex = Regex::new(r"^(\S+)\s*:\s*struct.begin(.*)").unwrap();
        let struct_end_regex: Regex = Regex::new(r"^struct.end").unwrap();
        let value_regex: Regex = Regex::new(r"^(\S+)\s*=\s*(.*)").unwrap();

        let mut current_struct_key: Option<DefaultKey> = None;

        for line in cfg_str.lines() {
            let line = line.trim();

            if let Some(captures) = struct_begin_regex.captures(line) {
                let name = captures.get(1).unwrap().as_str().to_string();
                let meta = captures.get(2).unwrap().as_str().to_string();

                let struct_key = structs.insert(GscCfgStruct {
                    name: name.clone(), // cloning because we might need it below for the CfgValue
                    meta,
                    values: Vec::new(),
                    parent: current_struct_key,
                });

                if current_struct_key.is_none() {
                    root_structs.push(struct_key);
                } else {
                    let current_struct = structs
                        .get_mut(current_struct_key.expect("We handle none case above"))
                        .expect("Structs are never deleted");

                    current_struct.values.push(GscCfgValue {
                        name,
                        value: None,
                        struct_key: Some(struct_key),
                    });
                }

                current_struct_key = Some(struct_key);
                continue;
            }

            if struct_end_regex.is_match(line) {
                let current_struct = structs
                    .get_mut(current_struct_key.expect(
                        "By the time we get to struct.end, we should always have a current struct key",
                    ))
                    .expect("Structs are never deleted");

                current_struct_key = current_struct.parent;
                continue;
            }

            if let Some(captures) = value_regex.captures(line) {
                let name = captures.get(1).unwrap().as_str().to_string();
                let value = captures.get(2).unwrap().as_str().to_string();

                let current_struct = structs
                    .get_mut(current_struct_key.expect(
                        "By the time we get to a value, we should always have a current struct key",
                    ))
                    .expect("Structs are never deleted");

                current_struct.values.push(GscCfgValue {
                    name,
                    value: Some(value),
                    struct_key: None,
                });
            }
        }

        Self {
            name,
            root_structs,
            structs,
        }
    }
}

pub fn merge_cfg_structs(base: &GscCfg, our: &GscCfg, their: &GscCfg) -> GscCfg {
    let base_json = serde_json::to_value(&base).unwrap().to_string();
    let our_json = serde_json::to_value(&our).unwrap().to_string();
    let their_json = serde_json::to_value(&their).unwrap().to_string();

    serde_json::from_str::<GscCfg>(
        &merge::merge_json_strings(&base_json, &our_json, &their_json).unwrap(),
    )
    .unwrap()
}
