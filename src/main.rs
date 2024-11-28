extern crate git2;

use std::{
    fs::{self, File},
    io::BufWriter,
};

use git2::Repository;
use path_slash::PathExt as _;

pub mod git;
pub mod gsc_cfg;
pub mod merge;

fn unpak_pak(path: &std::path::Path, output_dir: &std::path::Path) {
    let pak = repak::PakBuilder::new()
        .reader(&mut std::io::BufReader::new(File::open(path).unwrap()))
        .unwrap();

    // Extract each file
    for entry_path in pak.files() {
        let out_path = output_dir
            .join(pak.mount_point().replace("../../../", ""))
            .join(&entry_path);
        println!("Extracting {} to {}", entry_path, out_path.display());

        // Create parent directories
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        // Extract the file
        pak.read_file(
            &entry_path,
            &mut std::io::BufReader::new(File::open(path).unwrap()),
            &mut fs::File::create(&out_path).unwrap(),
        )
        .unwrap();
    }
}

fn process_all_input_dirs(input_dir: &std::path::Path, repo: &Repository) {
    fn process_dir(dir: &std::path::Path, root_dir: &std::path::Path, repo: &Repository) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                process_dir(&path, root_dir, repo);
            } else {
                // Path relative to raw dir
                let relative_path = path.strip_prefix(root_dir).unwrap();

                // Create parent directories
                if let Some(parent) = repo.path().parent().unwrap().join(relative_path).parent() {
                    fs::create_dir_all(parent).unwrap();
                }

                println!(
                    "Copying {} to {}",
                    path.display(),
                    repo.path().parent().unwrap().display()
                );

                // If the file is a JSON file, normalize it
                if path.extension().map_or(false, |ext| ext == "json") {
                    println!("Normalizing JSON file: {}", path.display());
                    let content = std::fs::read_to_string(&path).unwrap();
                    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
                    std::fs::write(
                        repo.path().parent().unwrap().join(relative_path),
                        serde_json::to_string_pretty(&json).unwrap(),
                    )
                    .unwrap();
                } else {
                    // Copy the file to the modpack directory
                    std::fs::copy(
                        path.as_path(),
                        repo.path().parent().unwrap().join(relative_path),
                    )
                    .unwrap();
                }
            }
        }
    }

    let mut branches: Vec<String> = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir(input_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    // Sort entries by name, this will ensure decrease_health comes before decrease_health_again
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for entry in entries {
        let path = entry.path();
        let branch_name: String = path.file_name().unwrap().to_str().unwrap().to_string();

        println!("Processing branch: {}", branch_name);

        // First add untracked files to master
        git::checkout_branch(repo, "master").expect("Failed to checkout master");

        if path.is_dir() {
            process_dir(&path, &path, repo);
        } else if path.extension().map_or(false, |ext| ext == "pak") {
            unpak_pak(&path, &repo.path().parent().unwrap());
        } else {
            panic!("Unknown file type: {}", path.display());
        }

        git::commit_files(repo, &branch_name, true).expect("Failed to commit untracked_files");

        // Now checkout branch for this root dir and add tracked files
        println!("Checking out branch: {}", branch_name);
        git::checkout_branch(repo, &branch_name).expect("Failed to checkout branch");
        git::commit_files(repo, &branch_name, false).expect("Failed to commit tracked files");

        branches.push(branch_name);
    }

    // Merge all branches into master
    for branch in branches {
        git::checkout_branch(repo, "master").expect("Failed to checkout master");
        git::merge_branch(repo, &branch, git::MergeStrategy::Custom)
            .expect("Failed to merge branch");
    }
}

fn main() {
    use toml::Table;

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <config-file>", args[0]);
        std::process::exit(1);
    }

    let config_contents = std::fs::read_to_string(&args[1]).expect("Failed to read config file");
    let config = config_contents.parse::<Table>().unwrap();

    // Get the config file's directory
    let config_path = std::path::Path::new(&args[1]);
    let config_dir = config_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    // Join the config directory with staging_dir to get the full path
    let staging_dir = config["staging_dir"].as_str().unwrap();
    let full_staging_dir = config_dir.join(staging_dir);

    // Delete the modpack directory if it exists
    if full_staging_dir.exists() {
        std::fs::remove_dir_all(full_staging_dir.clone())
            .expect("Failed to delete modpack directory");
    }

    std::fs::create_dir_all(full_staging_dir.clone()).expect("Failed to create modpack directory");

    let repo: Repository = git::init_repository(full_staging_dir.to_str().unwrap())
        .expect("Failed to initialize modpack repository");

    let input_dir = config["input_dir"].as_str().unwrap();
    let full_input_dir = config_dir.join(input_dir);

    process_all_input_dirs(&full_input_dir, &repo);

    let name = config["name"].as_str().unwrap();
    let pak_path = config_dir.join(format!("{}.pak", name));
    let mut pak = repak::PakBuilder::new().writer(
        BufWriter::new(File::create(pak_path).expect("Failed to create pak file")),
        repak::Version::V8B,
        "../../../".to_string(),
        None,
    );

    fn collect_pak_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.file_name().unwrap() == ".git" {
                continue;
            }

            if path.is_dir() {
                collect_pak_files(&path, files);
            } else {
                files.push(path);
            }
        }
    }

    let mut pak_files = Vec::new();
    collect_pak_files(&full_staging_dir, &mut pak_files);

    for path in pak_files {
        let pak_path = path.strip_prefix(&full_staging_dir).unwrap();
        let path_slash = pak_path.to_slash().unwrap();
        println!("Adding {} to pak", path_slash);
        pak.write_file(&path_slash, fs::read(&path).unwrap())
            .unwrap();
    }

    pak.write_index().unwrap();
}
