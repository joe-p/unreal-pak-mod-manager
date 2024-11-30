use crate::merge;
use nom::{
    bytes::complete::{tag, take_till},
    character::complete::{multispace0, not_line_ending},
    IResult,
};
use serde::Deserialize;
use serde::Serialize;
use slotmap::DefaultKey;
use slotmap::SlotMap;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug)]
pub struct Stalker2CfgStruct {
    pub name: String,
    pub meta: String,
    pub values: Vec<Stalker2CfgValue>,
    pub parent: Option<DefaultKey>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stalker2CfgValue {
    pub name: String,
    pub value: Option<String>,
    pub struct_key: Option<DefaultKey>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stalker2Cfg {
    name: String,
    structs: SlotMap<DefaultKey, Stalker2CfgStruct>,
    root_values: Vec<Stalker2CfgValue>,
}

impl Display for Stalker2Cfg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for value in &self.root_values {
            if let Some(value_str) = &value.value {
                writeln!(f, "{} = {}", value.name, value_str)?;
            } else if let Some(struct_key) = value.struct_key {
                write!(f, "{}", self.struct_to_string(struct_key, 0))?;
            }
        }
        Ok(())
    }
}

impl Stalker2Cfg {
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

    pub fn from_str(name: String, cfg_str: &str) -> anyhow::Result<Self> {
        let mut root_values: Vec<Stalker2CfgValue> = Vec::new();
        let mut structs: SlotMap<DefaultKey, Stalker2CfgStruct> = SlotMap::new();
        let mut current_struct_key: Option<DefaultKey> = None;
        let mut struct_depth = 0;
        let mut line_number: i32 = 0;

        // Parser combinators
        fn struct_begin(input: &str) -> IResult<&str, (String, String)> {
            let (input, _) = multispace0(input)?;
            let (input, name) = take_till(|c: char| c == ':')(input)?;
            let (input, _) = tag(":")(input)?;
            let (input, _) = multispace0(input)?;
            let (input, _) = tag("struct.begin")(input)?;
            let (input, meta) = not_line_ending(input)?;
            Ok((input, (name.trim().to_string(), meta.to_string())))
        }

        fn struct_end(input: &str) -> IResult<&str, ()> {
            let (input, _) = multispace0(input)?;
            let (input, _) = tag("struct.end")(input)?;
            Ok((input, ()))
        }

        fn value_line(input: &str) -> IResult<&str, (String, String)> {
            let (input, _) = multispace0(input)?;
            let (input, name) = take_till(|c: char| c == '=')(input)?;
            let (input, _) = tag("=")(input)?;
            let (input, _) = multispace0(input)?;
            let (input, value) = not_line_ending(input)?;
            Ok((input, (name.trim().to_string(), value.to_string())))
        }

        for line in cfg_str.lines() {
            line_number += 1;

            if line.trim().starts_with("//") {
                continue;
            }

            if let Ok((_, (name, meta))) = struct_begin(line) {
                struct_depth += 1;
                let struct_key = structs.insert(Stalker2CfgStruct {
                    name: name.clone(),
                    meta,
                    values: Vec::new(),
                    parent: current_struct_key,
                });

                if current_struct_key.is_none() {
                    root_values.push(Stalker2CfgValue {
                        name,
                        value: None,
                        struct_key: Some(struct_key),
                    });
                } else {
                    let current_struct = structs
                        .get_mut(current_struct_key.expect("We handle none case above"))
                        .expect("Structs are never deleted");

                    current_struct.values.push(Stalker2CfgValue {
                        name,
                        value: None,
                        struct_key: Some(struct_key),
                    });
                }

                current_struct_key = Some(struct_key);
                continue;
            }

            if struct_end(line).is_ok() {
                struct_depth -= 1;
                if struct_depth < 0 {
                    return Err(anyhow::anyhow!(
                        "Found struct.end without matching struct.begin at line {}",
                        line_number
                    ));
                }

                let current_struct = structs
                    .get_mut(current_struct_key.expect(
                        "By the time we get to struct.end, we should always have a current struct key",
                    ))
                    .expect("Structs are never deleted");

                current_struct_key = current_struct.parent;
                continue;
            }

            if let Ok((_, (name, value))) = value_line(line) {
                if current_struct_key.is_none() {
                    root_values.push(Stalker2CfgValue {
                        name,
                        value: Some(value),
                        struct_key: None,
                    });
                } else {
                    let current_struct = structs
                        .get_mut(current_struct_key.expect(
                            "By the time we get to a value, we should always have a current struct key",
                        ))
                        .expect("Structs are never deleted");

                    current_struct.values.push(Stalker2CfgValue {
                        name,
                        value: Some(value),
                        struct_key: None,
                    });
                }
            }

            if line.contains("struct.begin") {
                return Err(anyhow::anyhow!(
                    "Unprocessed struct.begin statement at line {}: {}",
                    line_number,
                    line
                ));
            }

            if line.contains("struct.end") {
                return Err(anyhow::anyhow!(
                    "Unprocessed struct.end statement at line {}: {}",
                    line_number,
                    line
                ));
            }
        }

        if struct_depth > 0 {
            return Err(anyhow::anyhow!(
                "Found {} unclosed struct.begin statements at end of file",
                struct_depth
            ));
        }

        Ok(Self {
            name,
            root_values,
            structs,
        })
    }
}

pub fn merge_cfg_structs(
    base: &Stalker2Cfg,
    our: &Stalker2Cfg,
    their: &Stalker2Cfg,
) -> anyhow::Result<Stalker2Cfg> {
    let base_json = serde_json::to_value(&base)?.to_string();
    let our_json = serde_json::to_value(&our)?.to_string();
    let their_json = serde_json::to_value(&their)?.to_string();

    Ok(serde_json::from_str::<Stalker2Cfg>(
        &merge::merge_json_strings(&base_json, &our_json, &their_json)?,
    )?)
}
