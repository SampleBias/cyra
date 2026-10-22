use cyra_core::artifact::{ArtifactStore, canonical_json, sha256};
use serde_json::json;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::{
    fs,
    io::Write,
    path::Path,
    sync::{Arc, Barrier},
};

#[test]
fn a00_06_canonical_content_hash_is_stable_and_changes_with_content() {
    let left = canonical_json(&json!({"z":1,"a":{"y":2,"b":[true,null]}})).unwrap();
    let right =
        canonical_json(&serde_json::from_str(r#"{"a":{"b":[true,null],"y":2},"z":1}"#).unwrap())
            .unwrap();
    assert_eq!(left, right);
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_ne!(sha256(&left), sha256(b"abc"));
}

#[test]
fn a00_06_atomic_publication_refuses_overwrite_and_detects_tampering() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let store = ArtifactStore::create(&root).unwrap();
    let value = json!({"origin":"synthetic_fixture","x":1});
    let reference = store.put_json(&value).unwrap();
    assert_eq!(
        store.read_verified(&reference).unwrap(),
        canonical_json(&value).unwrap()
    );
    assert_eq!(
        store.put_json(&value).unwrap_err().kind(),
        std::io::ErrorKind::AlreadyExists
    );
    fs::write(root.join(&reference.relative_path), b"tampered").unwrap();
    assert_eq!(
        store.read_verified(&reference).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
}

#[test]
fn a00_06_cancelled_staging_leaves_no_published_partial_result() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let store = ArtifactStore::create(&root).unwrap();
    {
        let mut staged = store.stage().unwrap();
        staged.write_all(b"incomplete").unwrap();
    }
    assert_eq!(fs::read_dir(root).unwrap().count(), 0);
}

#[test]
fn a00_06_abrupt_process_exit_never_publishes_staging() {
    const CHILD_PATH: &str = "CYRA_TEST_STAGE_DIR";
    if let Some(path) = std::env::var_os(CHILD_PATH) {
        let store = ArtifactStore::create(Path::new(&path)).unwrap();
        let mut stage = store.stage().unwrap();
        stage.write_all(b"incomplete scientific artifact").unwrap();
        // Deliberately exit without running Drop, like an interrupted process.
        std::process::exit(123);
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "a00_06_abrupt_process_exit_never_publishes_staging",
        ])
        .env(CHILD_PATH, &root)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(123));
    let names: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names.len(), 1);
    assert!(names[0].to_string_lossy().starts_with(".stage-"));
    let store = ArtifactStore::create(&root).unwrap();
    let complete = store.put_json(&json!({"complete":true})).unwrap();
    assert!(store.read_verified(&complete).is_ok());
}

#[test]
fn a00_06_concurrent_identical_writes_have_exactly_one_winner() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let barrier = Arc::new(Barrier::new(4));
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let root = root.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let store = ArtifactStore::create(&root).unwrap();
                barrier.wait();
                store.put_json(&json!({"x":1})).is_ok()
            })
        })
        .collect();
    let successes = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .filter(|ok| *ok)
        .count();
    assert_eq!(successes, 1);
    assert_eq!(fs::read_dir(root).unwrap().count(), 1);
}

#[test]
fn a00_06_bundle_traversal_and_symlink_access_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let store = ArtifactStore::create(&root).unwrap();
    fs::write(temp.path().join("outside"), b"private").unwrap();
    symlink(temp.path().join("outside"), root.join("link")).unwrap();
    symlink(temp.path(), root.join("directory-link")).unwrap();
    for path in [
        "../outside",
        "/etc/passwd",
        "link",
        "directory-link/outside",
    ] {
        assert!(store.read_relative(Path::new(path)).is_err(), "{path}");
    }
    assert!(ArtifactStore::create(&root.join("directory-link/new")).is_err());
}

#[test]
fn a00_06_directory_descriptor_survives_path_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let held = temp.path().join("held");
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    let store = ArtifactStore::create(&root).unwrap();
    fs::rename(&root, &held).unwrap();
    symlink(&outside, &root).unwrap();
    let reference = store.put_json(&json!({"safe":true})).unwrap();
    assert!(held.join(reference.relative_path).is_file());
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}

#[test]
fn a00_07_private_files_and_directories_without_chmod_of_existing_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("nested/store");
    let store = ArtifactStore::create(&root).unwrap();
    let reference = store.put_json(&json!({"private":true})).unwrap();
    assert_eq!(
        fs::metadata(&root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(root.join(reference.relative_path))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(ArtifactStore::create(&root).is_err());
    assert_eq!(
        fs::metadata(root).unwrap().permissions().mode() & 0o777,
        0o755
    );
}
