use serde::{Deserialize, Serialize};
use sha2::digest::common::hazmat::{SerializableState, SerializedState};
use sha2::{Digest, Sha256};
use std::io;

use super::{denied, invalid};
use crate::FilePolicy;

/// A bounded checkpoint for one data file. Digest state format is version-pinned.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub format_version: u32,
    pub created_at: u64,
    pub retain_until: u64,
    pub lock_offset: u64,
    pub checksum: String,
    pub sha256_state: Vec<u8>,
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl Record {
    pub(crate) fn new(now: u64, retention: u64) -> io::Result<Self> {
        let policy = FilePolicy::new(now, retention).map_err(|_| invalid("Retention overflow"))?;
        let hasher = Sha256::new();
        Ok(Self {
            format_version: 1,
            created_at: now,
            retain_until: policy.retain_until(),
            lock_offset: 0,
            checksum: hex(&hasher.clone().finalize()),
            sha256_state: hasher.serialize().to_vec(),
        })
    }

    pub(crate) fn hasher(&self) -> io::Result<Sha256> {
        if self.format_version != 1
            || self.retain_until < self.created_at
            || self.lock_offset > u64::MAX / 8
        {
            return Err(invalid("Invalid metadata header"));
        }
        let state: SerializedState<Sha256> = self
            .sha256_state
            .as_slice()
            .try_into()
            .map_err(|_| invalid("Invalid SHA-256 state length"))?;
        // sha2 0.11 serialization: 32 state bytes, 8-byte LE block count,
        // followed by the block buffer position and pending bytes.
        let blocks = u64::from_le_bytes(state[32..40].try_into().unwrap());
        let position = u64::from(state[40]);
        if blocks.checked_mul(64).and_then(|n| n.checked_add(position)) != Some(self.lock_offset) {
            return Err(invalid("SHA-256 state length does not match LOCK"));
        }
        let hasher = Sha256::deserialize(&state).map_err(|_| invalid("Invalid SHA-256 state"))?;
        if hex(&hasher.clone().finalize()) != self.checksum {
            return Err(invalid("Checksum does not match SHA-256 state"));
        }
        Ok(hasher)
    }

    pub(crate) fn append(&self, offset: u64, data: &[u8]) -> io::Result<Self> {
        if offset != self.lock_offset {
            return Err(denied("Append offset must equal LOCK"));
        }
        let lock_offset = offset
            .checked_add(data.len() as u64)
            .filter(|n| *n <= u64::MAX / 8)
            .ok_or_else(|| invalid("SHA-256 length overflow"))?;
        let mut hasher = self.hasher()?;
        hasher.update(data);
        Ok(Self {
            lock_offset,
            checksum: hex(&hasher.clone().finalize()),
            sha256_state: hasher.serialize().to_vec(),
            ..self.clone()
        })
    }
}
