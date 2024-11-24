use regex::Regex;

pub fn parse_config(input: &str) {
    let lines = input.lines().collect::<Vec<&str>>();
    let mut structs: Vec<String> = Vec::new();
    // Create the regex pattern - matches word at start (^) followed by optional whitespace (\s*) and equals
    let assignment_re = Regex::new(r"^\w+\s*=").unwrap();

    for (line_num, line) in lines.iter().enumerate() {
        if line.trim().matches("struct.begin").count() > 0 {
            let struct_name = line.split(":").nth(0).unwrap().trim();
            structs.push(struct_name.to_string());
        } else if line.trim().matches("struct.end").count() > 0 {
            structs.pop().expect("Failed to pop. This appears to be an error in parsing the beginning or ending of a struct.");
        } else if assignment_re.is_match(line.trim()) {
            let name = line.trim().split("=").nth(0).unwrap().trim();
            let value = line.trim().split_once('=').unwrap().1.trim();
            println!(
                "{}: {}.{} = {}",
                line_num + 1,
                structs.join("."),
                name,
                value
            );
        }
    }
}
