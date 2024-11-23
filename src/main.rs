extern crate git2;

use git2::Repository;
use git2::Signature;

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
                // Copy the file to the modpack directory
                std::fs::copy(
                    path.as_path(),
                    repo.path().parent().unwrap().join(relative_path),
                )
                .unwrap();
            }
        }
    }

    for entry in std::fs::read_dir(raw_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            process_dir(&path, &path, repo);
        }

        // Get the files added to the repo
        let statuses = repo.statuses(None).unwrap();
        for status in statuses.iter() {
            println!("Is new?: {}", status.status().is_wt_new());
        }

        commit(repo, "Added new files");
    }
}

fn commit(repo: &Repository, commit_message: &str) -> git2::Oid {
    // Get the index
    let mut index = repo.index().expect("Failed to get index");

    // Add all files in the repository directory to the index
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .expect("Failed to add files to index");
    index.write().expect("Failed to write index");

    // Write the tree object
    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");

    // Create a signature for the commit
    let signature = Signature::now("Strelok", "The Zone").expect("Failed to create signature");

    // Get the parent commit if HEAD exists
    let parent_commits = match repo.head() {
        Ok(head) => vec![head.peel_to_commit().expect("Failed to get head commit")],
        Err(_) => vec![], // Empty vec for initial commit
    };

    let parent_commits: Vec<&git2::Commit> = parent_commits.iter().collect();

    // Create the commit
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &parent_commits, // Use parent commits instead of empty slice
    )
    .expect("Failed to commit")
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

    let repo: Repository = Repository::init(full_modpack_dir.clone())
        .expect("Failed to initialize modpack repository");

    println!("Repository: {}", repo.path().display());

    let commit_id = commit(&repo, "Initial commit");

    println!("Initial commit created: {}", commit_id);

    let raw_dir = config_dir.join(config["raw_dir"].as_str().unwrap());

    process_all_raw_dirs(&raw_dir, &repo);
}
