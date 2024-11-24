extern crate git2;

use git2::Repository;

pub mod cfg_parser;
pub mod git;
pub mod merge;

fn process_all_raw_dirs(raw_dir: &std::path::Path, repo: &Repository) {
    fn process_dir(dir: &std::path::Path, root_dir: &std::path::Path, repo: &Repository) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                process_dir(&path, root_dir, repo);
            } else {
                // Path relative to raw dir
                let relative_path = path.strip_prefix(root_dir).unwrap();

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

    for entry in std::fs::read_dir(raw_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let branch_name: String = path.file_name().unwrap().to_str().unwrap().to_string();

        println!("Processing branch: {}", branch_name);

        // First add untracked files to master
        git::checkout_branch(repo, "master").expect("Failed to checkout master");
        if path.is_dir() {
            process_dir(&path, &path, repo);
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
        git::merge_branch(repo, &branch).expect("Failed to merge branch");
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

    // Join the config directory with modpack_dir to get the full path
    let modpack_dir = config["modpack_dir"].as_str().unwrap();
    let full_modpack_dir = config_dir.join(modpack_dir);

    // Delete the modpack directory if it exists
    if full_modpack_dir.exists() {
        std::fs::remove_dir_all(full_modpack_dir.clone())
            .expect("Failed to delete modpack directory");
    }

    std::fs::create_dir_all(full_modpack_dir.clone()).expect("Failed to create modpack directory");

    let repo: Repository = git::init_repository(full_modpack_dir.to_str().unwrap())
        .expect("Failed to initialize modpack repository");

    let raw_dir = config["raw_dir"].as_str().unwrap();
    let full_raw_dir = config_dir.join(raw_dir);

    process_all_raw_dirs(&full_raw_dir, &repo);

    let config_str = std::fs::read_to_string(
        "/Users/joe/git/joe-p/unreal-pak-mod-manager/example/raw/add_mutant/bloodsucker.cfg",
    )
    .expect("Failed to read config file");
    cfg_parser::parse_config(&config_str)
}
