pub fn merge_json_strings(base: &str, ours: &str, theirs: &str) -> anyhow::Result<String> {
    // Parse the JSON strings into Value objects
    let mut base_json: serde_json::Value = serde_json::from_str(base)?;
    let our_json: serde_json::Value = serde_json::from_str(ours)?;
    let their_json: serde_json::Value = serde_json::from_str(theirs)?;

    // Merge the JSON values
    let our_diff = json_patch::diff(&base_json, &our_json);
    let their_diff = json_patch::diff(&base_json, &their_json);

    json_patch::patch(&mut base_json, &our_diff)?;
    json_patch::patch(&mut base_json, &their_diff)?;

    // Convert back to string
    Ok(serde_json::to_string_pretty(&base_json)?)
}
