use autobricks_worm::{EntryPolicy, FilePolicy, PolicyError};

#[test]
fn appends_preserve_creation_based_expiry() {
    let policy = FilePolicy::new(100, 365 * 86_400).unwrap();
    let updated = policy
        .after_append(0, 10)
        .unwrap()
        .after_append(10, 20)
        .unwrap();
    assert_eq!(updated.created_at(), 100);
    assert_eq!(updated.retain_until(), policy.retain_until());
    assert_eq!(updated.lock_offset(), 30);
}

#[test]
fn overwrite_and_holes_are_rejected_even_after_retention() {
    let policy = FilePolicy::new(100, 10)
        .unwrap()
        .after_append(0, 20)
        .unwrap();
    assert_eq!(policy.check_delete(110), Ok(()));
    assert_eq!(policy.after_append(0, 1), Err(PolicyError::NotAtEnd));
    assert_eq!(policy.after_append(21, 1), Err(PolicyError::NotAtEnd));
}

#[test]
fn deletion_changes_at_exact_expiry() {
    let policy = FilePolicy::new(100, 10).unwrap();
    assert_eq!(policy.check_delete(109), Err(PolicyError::RetentionActive));
    assert_eq!(policy.check_delete(110), Ok(()));
    assert_eq!(policy.check_delete(111), Ok(()));
}

#[test]
fn paths_remain_fixed_across_append_and_retention_expiry() {
    let file = FilePolicy::new(100, 10).unwrap();
    assert_eq!(file.check_delete(109), Err(PolicyError::RetentionActive));
    assert_eq!(
        EntryPolicy::File(file.clone()).check_rename(),
        Err(PolicyError::ImmutablePath)
    );

    let appended = file.after_append(0, 20).unwrap();
    assert_eq!(appended.check_delete(110), Ok(()));
    assert_eq!(
        EntryPolicy::File(appended).check_rename(),
        Err(PolicyError::ImmutablePath)
    );
    assert_eq!(
        EntryPolicy::Directory.check_rename(),
        Err(PolicyError::ImmutablePath)
    );
}

#[test]
fn overflow_cannot_wrap_expiry_or_lock() {
    assert_eq!(FilePolicy::new(u64::MAX, 1), Err(PolicyError::Overflow));
    let policy = FilePolicy::new(0, 0)
        .unwrap()
        .after_append(0, u64::MAX)
        .unwrap();
    assert_eq!(policy.after_append(u64::MAX, 1), Err(PolicyError::Overflow));
    assert_eq!(policy.lock_offset(), u64::MAX);
}
