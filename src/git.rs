use git2::{Error, FileFavor, MergeOptions, Repository};
use std::{io::Read, path::Path};

use crate::{gsc_cfg, merge};
use gsc_cfg::GscCfg;

pub fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<(), Error> {
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

    println!("Branch: {:?}", branch.name().unwrap());

    // Get the branch's reference
    let branch_ref = branch.get();

    // Get the commit that the branch points to
    let commit = repo.find_commit(branch_ref.target().unwrap())?;

    // Create a checkout builder and set options
    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    checkout_builder
        .allow_conflicts(true)
        .conflict_style_merge(true)
        .safe();

    // Perform the checkout
    repo.checkout_tree(commit.as_object(), Some(&mut checkout_builder))?;

    // Set HEAD to point to the new branch
    repo.set_head(branch_ref.name().unwrap())?;

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
    println!(
        "Merging branch: {} with strategy: {:?}",
        from_branch, strategy
    );
    // Get the source branch's commit
    let from = repo.find_branch(from_branch, git2::BranchType::Local)?;
    let from_commit = repo.find_commit(from.get().target().unwrap())?;

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
            if let Err(e) = handle_merge_conflict(repo, &path, ancestor_id, our_id, their_id) {
                println!("{}: Will try again...", e);

                // This keeps their file
                // repo.index().unwrap().remove_path(Path::new(&path))?;
                // repo.index().unwrap().write()?;

                // Overwrite the current file content with ours
                let our_blob = repo.find_blob(our_id)?;
                let workdir = repo.workdir().expect("Repository has no working directory");
                let full_path = workdir.join(&path);
                std::fs::write(&full_path, our_blob.content())
                    .expect("Failed to write restored file");

                repo.index().unwrap().add_path(Path::new(&path))?;
                repo.index().unwrap().write()?;

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
) -> Result<(), Error> {
    println!("Conflict in file: {}", path);

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
        return write_and_stage(repo, path, merged);
    }

    if path.ends_with(".cfg") {
        let base_cfg = GscCfg::from_str(path.to_string(), &base_buf);
        let our_cfg = GscCfg::from_str(path.to_string(), &our_buf);
        let their_cfg = GscCfg::from_str(path.to_string(), &their_buf);

        let merged_cfg = gsc_cfg::merge_cfg_structs(&base_cfg, &our_cfg, &their_cfg);

        return write_and_stage(repo, path, merged_cfg.to_string());
    }

    Err(Error::from_str(&format!(
        "Failed to resolve conflict for file: {}",
        path
    )))
}

pub fn commit_files(repo: &Repository, message: &str, only_new: bool) -> Result<(), Error> {
    let mut index = repo.index()?;
    let mut files_to_commit = false;

    // Add only untracked files
    let mut cb = |path: &Path, _matched_pathspec: &[u8]| -> i32 {
        let should_add = if only_new {
            repo.status_file(path).unwrap().is_wt_new()
        } else {
            true
        };

        if should_add {
            files_to_commit = true;
            println!("Adding file: {}", path.display());
            0 // Add the file
        } else {
            1 // Skip the file
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
    let commit = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;

    println!("Commit: {:?}", commit);

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
        let signature = git2::Signature::now("Strelok", "The Zone").unwrap();
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
