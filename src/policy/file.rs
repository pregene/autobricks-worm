use super::PolicyError;

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
