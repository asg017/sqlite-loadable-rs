use super::{file::FileLock, kind::LockKind, range::RangeLock, wrapper::Lock, *};
use std::{fs::File, ops::Range, path::PathBuf};

// NOTE: subsumed by LockKind
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// #[repr(u16)]
// pub enum WalIndexLock {
//     None = 1,
//     Shared,
//     Exclusive,
// }

pub struct Connection {
    path: PathBuf,
    file: File,
    file_ino: u64,
    lock: Option<Lock>,
}

pub struct WalConnection {
    pub(crate) path: PathBuf,
    pub(crate) file_lock: FileLock,
    pub(crate) wal_lock: RangeLock,
    pub(crate) readonly: bool,
}

pub trait WalIndex: Sync {
    // fn enabled() -> bool {
    //     true
    // }

    fn map(&mut self, region: u32) -> Result<[u8; 32768], std::io::Error>;

    fn lock(&mut self, locks: Range<u8>, lock: LockKind) -> Result<bool, std::io::Error>;

    fn delete(self) -> Result<(), std::io::Error>;

    fn pull(&mut self, _region: u32, _data: &mut [u8; 32768]) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn push(&mut self, _region: u32, _data: &[u8; 32768]) -> Result<(), std::io::Error> {
        Ok(())
    }
}
