use git2::{Error, FileFavor, MergeOptions, Repository};
use std::{io::Read, path::Path};

use crate::{merge, stalker2_cfg, unreal_ini};
use stalker2_cfg::Stalker2Cfg;
use unreal_ini::UnrealIni;

pub fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<(), Error> {
    let branch_name = &normalize_git_ref(branch_name);

    // Try to find the branch first
    let branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(branch) => branch,
        Err(_) => {
            // Branch doesn't exist, create it from HEAD
            let head = repo.head()?;
            let head_commit = head.peel_to_commit()?;
            repo.branch(branch_name, &head_commit, false)?
        }
    };

    // Get the branch's reference
    let branch_ref = branch.get();

    // Get the commit that the branch points to
    let commit = repo.find_commit(branch_ref.target().expect(
        "should always be able to get the target of the branch reference since we created it",
    ))?;

    // Create a checkout builder and set options
    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    checkout_builder
        .allow_conflicts(true)
        .conflict_style_merge(true)
        .safe();

    // Perform the checkout
    repo.checkout_tree(commit.as_object(), Some(&mut checkout_builder))?;

    // Set HEAD to point to the new branch
    repo.set_head(branch_ref.name().expect(
        "should always be able to get the name of the branch reference since we created it",
    ))?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergeStrategy {
    Custom,    // Use custom merge logic
    Theirs,    // Use theirs
    Overwrite, // Overwrite our version with theirs
}

pub fn merge_branch(
    repo: &Repository,
    from_branch: &str,
    strategy: MergeStrategy,
) -> Result<(), Error> {
    let from_branch = &normalize_git_ref(from_branch);

    println!("{}: Merging files", from_branch);

    // Get the source branch's commit
    let from = repo.find_branch(from_branch, git2::BranchType::Local)?;
    let from_commit = repo.find_commit(from.get().target().expect(
        "should always be able to get the target of the branch reference since we created it",
    ))?;

    // Get the current HEAD commit
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;

    // get annotated commit from the from_commit
    let annotated_commit = repo.find_annotated_commit(from_commit.id())?;

    // Set up merge options with custom conflict handling
    let mut merge_opts = MergeOptions::new();
    if strategy == MergeStrategy::Theirs {
        merge_opts.file_favor(FileFavor::Theirs);
    } else {
        merge_opts.file_favor(FileFavor::Normal);
    }

    // Perform the merge with options
    repo.merge(&[&annotated_commit], Some(&mut merge_opts), None)
        .expect("Failed to perform merge");

    // Get conflicted files
    let mut unhandled_conflicts = false;
    let index = repo.index()?;

    if index.conflicts()?.count() == 0 {
        println!("{}: All files merged without conflicts", from_branch);
    }

    for entry in index.conflicts()? {
        let conflict = entry?;

        // Extract file paths for conflicting versions
        let path = conflict
            .our
            .as_ref()
            .map(|e| String::from_utf8_lossy(&e.path).to_string());
        let our_id = conflict.our.as_ref().map(|e| e.id);
        let their_id = conflict.their.as_ref().map(|e| e.id);

        if let (Some(path), Some(our_id), Some(their_id)) = (path, our_id, their_id) {
            let ancestor_id = conflict
                .ancestor
                .as_ref()
                .map(|e| e.id)
                .expect("No ancestor");

            // Handle potential error from merge conflict resolution
            if let Err(_e) =
                handle_merge_conflict(repo, &path, ancestor_id, our_id, their_id, &from_branch)
            {
                // Overwrite the current file content with ours
                let our_blob = repo.find_blob(our_id)?;
                let workdir = repo.workdir().expect("Repository has no working directory");
                let full_path = workdir.join(&path);
                std::fs::write(&full_path, our_blob.content())
                    .expect("Failed to write restored file");

                let mut index = repo.index()?;
                index.add_path(Path::new(&path))?;
                index.write()?;

                unhandled_conflicts = true;

                continue;
            }
        }
    }

    // Create the merge commit
    let sig = repo.signature()?;
    let message = format!("Merge branch '{}'", from_branch);
    let tree = repo.index()?.write_tree()?;
    let tree = repo.find_tree(tree)?;

    // Don't include the from_commit in parents so we can merge again with a different strategy
    repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&head_commit])?;

    // Clean up the merge state
    repo.cleanup_state()?;

    if unhandled_conflicts {
        if strategy == MergeStrategy::Theirs {
            merge_branch(repo, from_branch, MergeStrategy::Overwrite)?;
        } else {
            merge_branch(repo, from_branch, MergeStrategy::Theirs)?;
        }
    }

    Ok(())
}

fn handle_merge_conflict(
    repo: &Repository,
    path: &str,
    base_id: git2::Oid,
    our_id: git2::Oid,
    their_id: git2::Oid,
    mod_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_blob = repo.find_blob(base_id)?;
    let our_blob = repo.find_blob(our_id)?;
    let their_blob = repo.find_blob(their_id)?;

    let mut base_buf = String::new();
    base_blob
        .content()
        .read_to_string(&mut base_buf)
        .expect("Failed to read base blob");

    let mut our_buf = String::new();
    our_blob
        .content()
        .read_to_string(&mut our_buf)
        .expect("Failed to read our blob");

    let mut their_buf = String::new();
    their_blob
        .content()
        .read_to_string(&mut their_buf)
        .expect("Failed to read their blob");

    // Helper function to write and stage merged content
    fn write_and_stage(repo: &Repository, path: &str, content: String) -> Result<(), Error> {
        let workdir = repo.workdir().expect("Repository has no working directory");
        let full_path = workdir.join(path);

        // Write the merged content to the file
        std::fs::write(&full_path, content + "\n").expect("Failed to write merged content");

        // Stage the merged file
        let mut index = repo.index()?;
        index.add_path(Path::new(path))?;
        index.write()?;

        Ok(())
    }

    if path.ends_with(".json") {
        let merged = merge::merge_json_strings(&base_buf, &our_buf, &their_buf)
            .expect("Failed to merge JSON");

        println!("{}: Merged JSON values in {}", mod_name, path);
        return Ok(write_and_stage(repo, path, merged)?);
    }

    if path.ends_with(".cfg") {
        let base_cfg = Stalker2Cfg::from_str(path.to_string(), &base_buf)?;
        let our_cfg = Stalker2Cfg::from_str(path.to_string(), &our_buf)?;
        let their_cfg = Stalker2Cfg::from_str(path.to_string(), &their_buf)?;

        let merged_cfg = stalker2_cfg::merge_cfg_structs(&base_cfg, &our_cfg, &their_cfg)?;

        println!("{}: Merged cfg values in {}", mod_name, path);
        return Ok(write_and_stage(repo, path, merged_cfg.to_string())?);
    }

    if path.ends_with(".ini") {
        let base_ini = UnrealIni::from_str(&base_buf);
        let our_ini = UnrealIni::from_str(&our_buf);
        let their_ini = UnrealIni::from_str(&their_buf);

        let merged_ini = unreal_ini::merge_unreal_inis(&base_ini, &our_ini, &their_ini)?;

        println!("{}: Merged ini values in {}", mod_name, path);
        return Ok(write_and_stage(repo, path, merged_ini.to_string())?);
    }

    Err(Box::new(Error::from_str(&format!(
        "Failed to resolve conflict for file: {}",
        path
    ))))
}

pub fn commit_files(repo: &Repository, message: &str, only_new: bool) -> Result<(), Error> {
    let mut index = repo.index()?;
    let mut files_to_commit = false;

    // Add only untracked files
    let mut cb = |path: &Path, _matched_pathspec: &[u8]| {
        if !only_new || repo.status_file(path).map_or(false, |s| s.is_wt_new()) {
            files_to_commit = true;
            0
        } else {
            1
        }
    };
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, Some(&mut cb))?;

    // Return early if no files to commit
    if !files_to_commit {
        return Ok(());
    }

    index.write()?;

    // Create tree from index
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Get the current HEAD commit as the parent
    let head = repo.head()?;
    let parent_commit = head.peel_to_commit()?;

    // Create the commit
    let signature = repo.signature()?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(())
}

pub fn init_repository(path: &str) -> Result<Repository, Error> {
    // Initialize a new repository
    let repo = Repository::init(path)?;

    {
        // Create new scope to ensure tree is dropped before we return repo
        // Create an empty tree for the initial commit
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };
        let tree = repo.find_tree(tree_id)?;

        // Create the initial commit
        let signature = git2::Signature::now("Strelok", "The Zone")?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[], // No parent commits for initial commit
        )?;
    } // tree is dropped here, releasing its borrow of repo

    Ok(repo)
}

pub fn normalize_git_ref(input: &str) -> String {
    let mut result = String::new();
    let mut last_char: Option<char> = None;

    for c in input.chars() {
        let replacement = match c {
            // Replace ASCII control characters, space, ~, ^, :, ?, *, [, \
            c if c < ' '
                || c == ' '
                || c == '~'
                || c == '^'
                || c == ':'
                || c == '?'
                || c == '*'
                || c == '['
                || c == '\\' =>
            {
                '_'
            }

            // Handle slash specially
            '/' => {
                if last_char.map_or(true, |lc| lc == '/') {
                    // Skip consecutive slashes
                    continue;
                }
                '/'
            }

            // Keep allowed characters
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' => c,

            // Replace everything else with underscore
            _ => '_',
        };

        // Special cases for dots
        if replacement == '.' {
            // Skip if it would create '..' sequence
            if last_char.map_or(false, |lc| lc == '.') {
                continue;
            }
            // Skip if it would start a component with '.'
            if last_char.map_or(false, |lc| lc == '/') {
                continue;
            }
        }

        result.push(replacement);
        last_char = Some(replacement);
    }

    // Post-processing
    let mut normalized = result
        .trim_matches('/') // Remove leading/trailing slashes
        .replace("@{", "__") // Replace @{ sequence
        .replace(".lock", "_lock") // Replace .lock at end of components
        .to_string();

    // Handle special case where result is just "@"
    if normalized == "@" {
        normalized = String::from("_");
    }

    // Remove trailing dots
    while normalized.ends_with('.') {
        normalized.pop();
    }

    normalized
}
