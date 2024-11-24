use git2::{Error, FileFavor, MergeOptions, Repository};
use std::{io::Read, path::Path};

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

// TODO: Custom merge driver for cfg files
pub fn merge_branch(repo: &Repository, from_branch: &str) -> Result<(), Error> {
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
    merge_opts.file_favor(FileFavor::Normal);

    // Perform the merge with options
    repo.merge(&[&annotated_commit], Some(&mut merge_opts), None)
        .expect("Failed to perform merge");

    // Get conflicted files
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
            handle_merge_conflict(repo, &path, our_id, their_id)?;
        }
    }

    // Create the merge commit
    let sig = repo.signature()?;
    let message = format!("Merge branch '{}'", from_branch);
    let tree = repo.index()?.write_tree()?;
    let tree = repo.find_tree(tree)?;

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &message,
        &tree,
        &[&head_commit, &from_commit],
    )?;

    // Clean up the merge state
    repo.cleanup_state()?;

    Ok(())
}

fn handle_merge_conflict(
    repo: &Repository,
    path: &str,
    our_id: git2::Oid,
    their_id: git2::Oid,
) -> Result<(), Error> {
    println!("Conflict in file: {}", path);

    let our_blob = repo.find_blob(our_id)?;
    let their_blob = repo.find_blob(their_id)?;

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

    println!("Our blob: {:?}", our_buf);
    println!("Their blob: {:?}", their_buf);

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