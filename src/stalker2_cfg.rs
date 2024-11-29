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
    root_structs: Vec<DefaultKey>,
}

impl Display for Stalker2Cfg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for struct_key in &self.root_structs {
            write!(f, "{}", self.struct_to_string(*struct_key, 0))?;
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

    pub fn from_str(name: String, cfg_str: &str) -> Stalker2Cfg {
        let mut root_structs: Vec<DefaultKey> = Vec::new();
        let mut structs: SlotMap<DefaultKey, Stalker2CfgStruct> = SlotMap::new();
        let mut current_struct_key: Option<DefaultKey> = None;

        // Parser combinators
        fn struct_begin(input: &str) -> IResult<&str, (String, String)> {
            let (input, _) = multispace0(input)?;
            let (input, name) = take_till(|c: char| c.is_whitespace())(input)?;
            let (input, _) = multispace0(input)?;
            let (input, _) = tag(":")(input)?;
            let (input, _) = multispace0(input)?;
            let (input, _) = tag("struct.begin")(input)?;
            let (input, meta) = not_line_ending(input)?;
            Ok((input, (name.to_string(), meta.to_string())))
        }

        fn struct_end(input: &str) -> IResult<&str, ()> {
            let (input, _) = multispace0(input)?;
            let (input, _) = tag("struct.end")(input)?;
            Ok((input, ()))
        }

        fn value_line(input: &str) -> IResult<&str, (String, String)> {
            let (input, _) = multispace0(input)?;
            let (input, name) = take_till(|c: char| c.is_whitespace())(input)?;
            let (input, _) = multispace0(input)?;
            let (input, _) = tag("=")(input)?;
            let (input, _) = multispace0(input)?;
            let (input, value) = not_line_ending(input)?;
            Ok((input, (name.to_string(), value.to_string())))
        }

        for line in cfg_str.lines() {
            if let Ok((_, (name, meta))) = struct_begin(line) {
                let struct_key = structs.insert(Stalker2CfgStruct {
                    name: name.clone(),
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
                let current_struct = structs
                    .get_mut(current_struct_key.expect(
                        "By the time we get to struct.end, we should always have a current struct key",
                    ))
                    .expect("Structs are never deleted");

                current_struct_key = current_struct.parent;
                continue;
            }

            if let Ok((_, (name, value))) = value_line(line) {
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

        Self {
            name,
            root_structs,
            structs,
        }
    }
}

pub fn merge_cfg_structs(
    base: &Stalker2Cfg,
    our: &Stalker2Cfg,
    their: &Stalker2Cfg,
) -> Result<Stalker2Cfg, Box<dyn std::error::Error>> {
    let base_json = serde_json::to_value(&base)?.to_string();
    let our_json = serde_json::to_value(&our)?.to_string();
    let their_json = serde_json::to_value(&their)?.to_string();

    Ok(serde_json::from_str::<Stalker2Cfg>(
        &merge::merge_json_strings(&base_json, &our_json, &their_json)?,
    )?)
}
