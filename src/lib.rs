//! In-memory WORM policy calculations.
//!
//! Timestamps are UTC Unix seconds supplied by the caller.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    Overflow,
    NotAtEnd,
    RetentionActive,
}

/// Private fields prevent callers from changing the creation time or retention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePolicy {
    created_at: u64,
    retain_until: u64,
    lock_offset: u64,
}

impl FilePolicy {
    pub fn new(created_at: u64, retention_seconds: u64) -> Result<Self, PolicyError> {
        let retain_until = created_at
            .checked_add(retention_seconds)
            .ok_or(PolicyError::Overflow)?;
        Ok(Self {
            created_at,
            retain_until,
            lock_offset: 0,
        })
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }
    pub fn retain_until(&self) -> u64 {
        self.retain_until
    }
    pub fn lock_offset(&self) -> u64 {
        self.lock_offset
    }

    /// Propose policy state for a complete, confirmed append.
    ///
    /// Returns a new policy with an advanced LOCK boundary and the same retention deadline.
    pub fn after_append(&self, offset: u64, length: u64) -> Result<Self, PolicyError> {
        if offset != self.lock_offset {
            return Err(PolicyError::NotAtEnd);
        }
        let lock_offset = offset.checked_add(length).ok_or(PolicyError::Overflow)?;
        Ok(Self {
            lock_offset,
            ..self.clone()
        })
    }

    /// Evaluate whole-file deletion eligibility against the retention deadline.
    pub fn check_delete(&self, now: u64) -> Result<(), PolicyError> {
        if now < self.retain_until {
            Err(PolicyError::RetentionActive)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn overflow_cannot_wrap_expiry_or_lock() {
        assert_eq!(FilePolicy::new(u64::MAX, 1), Err(PolicyError::Overflow));
        let policy = FilePolicy::new(0, 0)
            .unwrap()
            .after_append(0, u64::MAX)
            .unwrap();
        assert_eq!(policy.after_append(u64::MAX, 1), Err(PolicyError::Overflow));
        assert_eq!(policy.lock_offset(), u64::MAX);
    }
}
