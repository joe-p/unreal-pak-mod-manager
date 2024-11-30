extern crate git2;

use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use git2::Repository;
use path_slash::PathExt as _;

pub mod git;
pub mod merge;
pub mod stalker2_cfg;
pub mod unreal_ini;

#[derive(serde::Deserialize, Clone)]
struct UpmmModConfig {
    // The priority of the mod
    // Lower numbers are merged first, meaning changes in mod priority=2 will take priority over changes in mod priority=1
    // Without an explicit priority set, the mods priority is set via alphabetical order
    // For example, "a.pak", "b.pak", and "c.pak" will have priorities 0, 1, and 2 respectively
    // As such, it's recommended to set priorities above 1000 and below -1000 to ensure adding new mods won't affect existing priorities
    priority: Option<i64>,
}

#[derive(serde::Deserialize)]
struct UpmmConfig {
    // The name of the modpack
    name: String,

    // The directory where all of the files are staged before being added to the .pak file
    // This directory will be a git repository so you can use git to look at the history of the files
    // Each input mod will contain it's own branch and merge commit
    staging_dir: String,

    // The directory where the mods are located
    // The directory can contain either:
    // - Directories that are essentially unpacked .pak files (assumes default mount point of "../../../")
    // - .pak files
    mods_dir: String,

    // mods.<mod_name> allows you to set mod-specific options
    mods: Option<HashMap<String, UpmmModConfig>>,
}

const DEFAULT_CONFIG_FILE: &str = r#"
# The name of .pak file that is created
name = "upmm_modpack"

# All directories in this config are relative to the location of this config file

# The directory where all of the files are staged before being added to the .pak file
# This directory will be a git repository so you can use git to look at the history of the files
# Each input mod will contain it's own branch and merge commit
staging_dir = "staging"

# The directory where the mods are located
# The directory can contain either:
# - Directories that are essentially unpacked .pak files (assumes default mount point of "../../../")
# - .pak files
mods_dir = "mods"

# mods.<mod_name> allows you to set mod-specific options

# mods.<mod_name>.priority sets the order in which the mods are merged into the final mod pack
# Lower numbers are merged first, meaning changes in mod priority=2 will take priority over changes in mod priority=1
# Without an explicit priority set, the mods priority is set via alphabetical order
# For example, "a.pak", "b.pak", and "c.pak" will have priorities 0, 1, and 2 respectively
# As such, it's recommended to set priorities above 1000 and below -1000 to ensure adding new mods won't affect existing priorities

# [mods."zzzz_Grok_Boar-40pHP_P.pak"]
# priority = -1000 # Merge this mod first
"#;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(
        value_name = "CONFIG_FILE",
        help = "Path to the configuration file. If not given, assume config.toml in current directory. If config.toml is not found, create it."
    )]
    config_file: Option<String>,
}

fn unpak_pak(path: &std::path::Path, output_dir: &std::path::Path) -> Result<()> {
    let pak = repak::PakBuilder::new().reader(&mut std::io::BufReader::new(
        File::open(path)
            .with_context(|| format!("Failed to open pak file '{}'", path.display()))?,
    ))?;

    // Extract each file
    for entry_path in pak.files() {
        let relative_out_path =
            PathBuf::from(pak.mount_point().replace("../../../", "")).join(&entry_path);

        let out_path = output_dir.join(&relative_out_path);

        println!(
            "{}: Extracting {}",
            path.file_name()
                .expect("should be able to get filename from path")
                .to_str()
                .expect("should be able to get str from filename"),
            relative_out_path
                .to_str()
                .expect("should be able to get str from path"),
        );

        // Create parent directories
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory '{}'", parent.display()))?;
        }

        // Extract the file to a string first
        let mut content = Vec::new();
        pak.read_file(
            &entry_path,
            &mut std::io::BufReader::new(File::open(path)?),
            &mut content,
        )?;

        // Normalize and write the content
        let normalized = normalize_content(&out_path, &content)?;
        fs::write(&out_path, normalized).context(format!(
            "failed to write to {}",
            &out_path.to_str().context("Failed to get str from path")?
        ))?;
    }

    Ok(())
}

fn normalize_content(path: &std::path::Path, content: &Vec<u8>) -> Result<Vec<u8>> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            let str_content = String::from_utf8(content.to_vec())
                .context(format!("non-utf8 bytes found in {}", path.display()))?;
            let json: serde_json::Value = serde_json::from_str(&str_content)?;
            Ok(serde_json::to_string_pretty(&json)?.into_bytes())
        }
        Some("cfg") => {
            let str_content = String::from_utf8(content.to_vec())
                .context(format!("non-utf8 bytes found in {}", path.display()))?;

            let cfg = stalker2_cfg::Stalker2Cfg::from_str(
                path.file_name()
                    .expect("should always be able to get the filename from the path")
                    .to_str()
                    .expect("should always be able to get the str from the filename")
                    .to_string(),
                &str_content,
            )?;

            Ok(cfg.to_string().into_bytes())
        }
        _ => Ok(content.to_vec()),
    }
}

fn process_all_mods_dirs(
    mods_dir: &std::path::Path,
    repo: &Repository,
    config: &UpmmConfig,
) -> Result<()> {
    fn process_dir(
        dir: &std::path::Path,
        root_dir: &std::path::Path,
        repo: &Repository,
    ) -> Result<()> {
        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory '{}'", dir.display()))?
        {
            let path = entry
                .with_context(|| "Failed to read directory entry")?
                .path();

            if path.is_dir() {
                process_dir(&path, root_dir, repo)?;
            } else {
                // Path relative to raw dir
                let relative_path = path.strip_prefix(root_dir)?;
                let repo_parent = repo
                    .path()
                    .parent()
                    .expect("should always be able to get the parent of the repo path");

                // Create parent directories
                if let Some(parent) = repo_parent.join(relative_path).parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create directory '{}'", parent.display())
                    })?;
                }

                println!(
                    "{}: Copying {}",
                    root_dir
                        .file_name()
                        .expect("should always be able to get the filename from the root dir")
                        .to_str()
                        .expect("should always be able to get the filename from the root dir"),
                    relative_path.display(),
                );

                let content = normalize_content(
                    &path,
                    &std::fs::read(&path)
                        .context(format!("Failed to read file '{}'", path.display()))?,
                )?;

                std::fs::write(repo_parent.join(relative_path), content).context(format!(
                    "Failed to write file '{}'",
                    relative_path.display()
                ))?;
            }
        }

        Ok(())
    }

    let mut entries: Vec<_> = std::fs::read_dir(mods_dir)
        .with_context(|| format!("Failed to read mods directory '{}'", mods_dir.display()))?
        .filter_map(Result::ok)
        .collect();

    // Sort entries by name, this will ensure decrease_health comes before decrease_health_again
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let mut priority_map: HashMap<PathBuf, i64> = HashMap::new();
    let mut current_idx = 0;

    for entry in &entries {
        let path = entry.path();
        let mod_name = path
            .file_name()
            .expect("should always be able to get the filename from the path")
            .to_str()
            .expect("should always be able to get the str from the filename");

        let priority = config
            .mods
            .as_ref()
            .and_then(|v| v.get(mod_name))
            .and_then(|v| v.priority)
            .unwrap_or(current_idx);

        priority_map.insert(path, priority);
        current_idx += 1;
    }

    // Sort entries based on their priorities
    entries.sort_by_key(|entry| {
        priority_map
            .get(&entry.path())
            .expect("should always be able to get the priority from the priority map")
    });

    println!("Processing the mods in the following order:");
    for entry in &entries {
        let path = entry.path();
        let priority = priority_map
            .get(&path)
            .expect("should always be able to get the priority from the priority map");

        println!("{}: {}", priority, path.display());
    }

    for entry in &entries {
        let path = entry.path();
        let branch_name: String = path
            .file_name()
            .expect("should always be able to get the filename from the path")
            .to_str()
            .expect("should always be able to get the str from the filename")
            .to_string();

        // First add untracked files to master
        git::checkout_branch(repo, "master").expect("Failed to checkout master");

        if path.is_dir() {
            process_dir(&path, &path, repo)?;
        } else if path.extension().map_or(false, |ext| ext == "pak") {
            unpak_pak(
                &path,
                &repo
                    .path()
                    .parent()
                    .expect("should always be able to get the parent of the repo path"),
            )?;
        } else {
            panic!("Unknown file type: {}", path.display());
        }

        git::commit_files(repo, &branch_name, true).expect("Failed to commit untracked_files");

        // Now checkout branch for this root dir and add tracked files
        git::checkout_branch(repo, &branch_name).expect("Failed to checkout branch");
        git::commit_files(repo, &branch_name, false).expect("Failed to commit tracked files");
    }

    for entry in &entries {
        let path = entry.path();
        let priority = priority_map
            .get(&path)
            .expect("should always be able to get the priority from the priority map");

        let branch = git::normalize_git_ref(
            path.file_name()
                .expect("should always be able to get the filename from the path")
                .to_str()
                .expect("should always be able to get the str from the filename"),
        );

        println!("{}: Merging with priority {}", branch, priority);

        git::checkout_branch(repo, "master").expect("Failed to checkout master");
        git::merge_branch(repo, &branch, git::MergeStrategy::Custom)
            .expect("Failed to merge branch");
    }

    Ok(())
}

fn create_modpack(config_path: &std::path::Path) -> Result<()> {
    let config_contents = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file '{}'", config_path.display()))?;
    let config: UpmmConfig = toml::from_str(&config_contents)?;

    // Get the config file's directory
    let config_dir = config_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    // Now use config.staging_dir, config.name, etc. directly
    let full_staging_dir = config_dir.join(&config.staging_dir);

    // Delete the modpack directory if it exists
    if full_staging_dir.exists() {
        std::fs::remove_dir_all(full_staging_dir.clone())
            .with_context(|| format!("Failed to delete modpack directory"))?;
    }

    std::fs::create_dir_all(full_staging_dir.clone()).with_context(|| {
        format!(
            "Failed to create modpack directory '{}'",
            full_staging_dir.display()
        )
    })?;

    let repo: Repository = git::init_repository(
        full_staging_dir
            .to_str()
            .context("Failed to get staging dir str")?,
    )
    .expect("Failed to initialize modpack repository");

    let mods_dir = config.mods_dir.clone();
    let full_mods_dir = config_dir.join(mods_dir);

    if !full_mods_dir.exists() {
        std::fs::create_dir_all(&full_mods_dir).with_context(|| {
            format!(
                "Failed to create mods directory '{}'",
                full_mods_dir.display()
            )
        })?;
        let absolute_mods_dir: PathBuf = fs::canonicalize(&full_mods_dir).with_context(|| {
            format!(
                "Failed to get absolute path for '{}'",
                full_mods_dir.display()
            )
        })?;

        println!(
            "Created mods directory, put pak files here and run this program again to create a modpack: {}",
            absolute_mods_dir.display()
        );

        println!("Press Enter to exit...");
        std::io::stdin().read_line(&mut String::new())?;
        return Ok(());
    }

    if std::fs::read_dir(&full_mods_dir)?.count() == 0 {
        let absolute_mods_dir: PathBuf = fs::canonicalize(&full_mods_dir)?;

        println!("Mods directory is empty, put pak files here and run this program again to create a modpack: {}", absolute_mods_dir.display());
        println!("Press Enter to exit...");
        std::io::stdin().read_line(&mut String::new())?;
        return Ok(());
    }

    process_all_mods_dirs(&full_mods_dir, &repo, &config)
        .with_context(|| "Failed to process all input directories")?;

    let name = config.name;
    let pak_path = config_dir.join(format!("{}.pak", name));
    let pak_name = pak_path
        .file_name()
        .context("Failed to get pak file name")?
        .to_str()
        .context("Failed to get file str")?
        .to_owned();
    let mut pak = repak::PakBuilder::new().writer(
        BufWriter::new(File::create(&pak_path).expect("Failed to create pak file")),
        repak::Version::V8B,
        "../../../".to_string(),
        None,
    );

    fn collect_pak_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir).context(format!("Failed to read dir {}", dir.display()))? {
            let entry = entry;
            let path = entry?.path();

            if path.file_name().context("Failed to get filename")? == ".git" {
                continue;
            }

            if path.is_dir() {
                collect_pak_files(&path, files)?;
            } else {
                files.push(path);
            }
        }

        Ok(())
    }

    let mut pak_files = Vec::new();
    collect_pak_files(&full_staging_dir, &mut pak_files)?;

    for path in pak_files {
        let file_path = path.strip_prefix(&full_staging_dir)?;
        let path_slash = file_path.to_slash().context("Failed to get slash")?;
        println!("{}: Packing {}", pak_name, path_slash);
        pak.write_file(
            &path_slash,
            fs::read(&path).expect(&format!("Failed to read {}", path.display())),
        )
        .expect(&format!(
            "Failed to write {} to {}",
            path_slash,
            path.display()
        ));
    }

    pak.write_index()?;

    println!("{} created successfully!", pak_path.display());
    Ok(())
}

fn run() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let config_path = match args.config_file {
        None => {
            let default_path = PathBuf::from("config.toml");
            if !default_path.exists() {
                std::fs::write(&default_path, DEFAULT_CONFIG_FILE)
                    .context("Failed to write default config file")?;

                let absolute_path = fs::canonicalize(&default_path)
                    .context("Failed to get absolute path of default config file")?;
                println!("Created default config file: {}", absolute_path.display());
            }

            default_path
        }
        Some(path) => PathBuf::from(path),
    };

    create_modpack(&config_path)?;

    Ok(())
}

fn main() {
    let result = run();

    if let Err(e) = &result {
        eprintln!("Error: {:#}", e);
    }

    println!("Press Enter to exit...");
    std::io::stdin().read_line(&mut String::new()).unwrap();

    if result.is_err() {
        std::process::exit(1);
    }
}
