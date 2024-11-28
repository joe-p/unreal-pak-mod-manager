use indexmap::IndexMap;
use nom::{
    bytes::complete::{tag, take_till},
    character::complete::{multispace0, not_line_ending},
    IResult,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fmt::Formatter;

use crate::merge;

#[derive(Debug, Serialize, Deserialize)]
pub struct UnrealIni {
    sections: IndexMap<String, UnrealIniSection>,
}

impl Display for UnrealIni {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (section_name, section) in &self.sections {
            writeln!(f, "[{}]", section_name)?;

            for (key, value) in &section.values {
                writeln!(f, "{} = {}", key, value)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UnrealIniSection {
    values: IndexMap<String, String>,
}

impl UnrealIni {
    pub fn from_str(s: &str) -> Self {
        let mut sections = IndexMap::new();

        fn new_section(input: &str) -> IResult<&str, (String, IndexMap<String, String>)> {
            let (input, _) = multispace0(input)?;
            let (input, _) = tag("[")(input)?;
            let (input, name) = take_till(|c| c == ']')(input)?;
            let (input, _) = tag("]")(input)?;

            Ok((input, (name.to_string(), IndexMap::new())))
        }

        fn value(input: &str) -> IResult<&str, (String, String)> {
            println!("Looking for value in: {}", input);
            let (input, _) = multispace0(input)?;
            let (input, key) = take_till(|c: char| c.is_whitespace() || c == '=')(input)?;
            let (input, _) = multispace0(input)?;
            let (input, _) = tag("=")(input)?;
            let (input, _) = multispace0(input)?;
            let (input, value) = not_line_ending(input)?;

            println!("Found value: {} = {}", key, value);

            Ok((input, (key.to_string(), value.to_string())))
        }

        let mut current_section = String::new();

        for line in s.lines() {
            let line = line.trim();

            if line.starts_with(';') {
                continue;
            }

            if let Ok((_, (name, values))) = new_section(line) {
                sections.insert(name.clone(), UnrealIniSection { values });
                current_section = name.clone();
            } else if let Ok((_, (key, value))) = value(line) {
                sections
                    .get_mut(&current_section)
                    .unwrap()
                    .values
                    .insert(key, value);
            }
        }

        Self { sections }
    }
}

pub fn merge_unreal_inis(base: &UnrealIni, our: &UnrealIni, their: &UnrealIni) -> UnrealIni {
    let base_json = serde_json::to_value(&base).unwrap().to_string();
    let our_json = serde_json::to_value(&our).unwrap().to_string();
    let their_json = serde_json::to_value(&their).unwrap().to_string();

    serde_json::from_str::<UnrealIni>(
        &merge::merge_json_strings(&base_json, &our_json, &their_json).unwrap(),
    )
    .unwrap()
}
