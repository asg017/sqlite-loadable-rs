use std::borrow::Cow;
use std::fs::{self, File, Permissions, OpenOptions as FsOpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::lock::file::FileLock;
use crate::lock::kind::LockKind;
use crate::lock::range::RangeLock;
use crate::lock::traits::{DatabaseHandle, OpenAccess, OpenKind, OpenOptions, Open};
use crate::lock::wal::WalIndex;
use crate::lock::wrapper::Lock;

/// [Vfs] test implementation based on Rust's [std::fs:File]. This implementation is not meant for
/// any use-cases except running SQLite unit tests, as the locking is only managed in process
/// memory.
#[derive(Default)]
pub struct TestVfs {
    temp_counter: AtomicUsize,
}

pub struct Connection {
    path: PathBuf,
    file: File,
    file_ino: u64,
    lock: Option<Lock>,
}

pub struct WalConnection {
    path: PathBuf,
    file_lock: FileLock,
    wal_lock: RangeLock,
    readonly: bool,
}

impl Open for TestVfs {
    type Handle = Connection;

    fn open(&self, db: &str, opts: OpenOptions) -> Result<Self::Handle, std::io::Error> {
        let path = normalize_path(Path::new(&db));
        if path.is_dir() {
            return Err(io::Error::new(ErrorKind::Other, "cannot open directory"));
        }

        let mut o = fs::OpenOptions::new();
        o.read(true).write(opts.access != OpenAccess::Read);
        let is_create = match opts.access {
            OpenAccess::Create => {
                o.create(true);
                true
            }
            OpenAccess::CreateNew => {
                o.create_new(true);
                true
            }
            _ => false,
        };
        let file = o.open(&path)?;
        let metadata = file.metadata()?;
        let file_ino = metadata.ino();

        if is_create && matches!(opts.kind, OpenKind::Wal | OpenKind::MainJournal) {
            if let Ok(mode) = permissions(&path) {
                fs::set_permissions(&path, Permissions::from_mode(mode)).ok();
            }
        }

        if opts.kind == OpenKind::Wal {
            // ensure wal index access
            let path = path.with_extension(format!(
                "{}-shm",
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.split_once('-').map(|(f, _)| f).unwrap_or(ext))
                    .unwrap_or("db")
            ));
            if path.exists()
                && fs::metadata(&path)
                    .map(|m| m.permissions().mode())
                    .unwrap_or(0o100000)
                    <= 0o100000
            {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "cannot read .db-shm file",
                ));
            }
        }

        Ok(Connection {
            path,
            // Lock needs to be created right away to ensure there is a free file descriptor for the
            // additional lock file.
            lock: if opts.kind == OpenKind::MainDb {
                Some(Lock::from_file(&file)?)
            } else {
                None
            },
            file,
            file_ino,
        })
    }
    
    /* 
    fn delete(&self, db: &str) -> Result<(), std::io::Error> {
        let path = normalize_path(Path::new(&db));
        fs::remove_file(path)
    }

    fn exists(&self, db: &str) -> Result<bool, std::io::Error> {
        Ok(Path::new(db).is_file())
    }

    fn access(&self, db: &str, write: bool) -> Result<bool, std::io::Error> {
        let metadata = fs::metadata(db)?;
        let readonly = metadata.permissions().readonly();
        Ok(!write || (write && !readonly))
    }
    */

    /// Required by vfs open + file_control
    fn temporary_name(&self) -> String {
        std::env::temp_dir()
            .join(format!(
                "etilqs_{:x}_{:x}.db",
                std::process::id(),
                self.temp_counter.fetch_add(1, Ordering::AcqRel),
            ))
            .to_string_lossy()
            .to_string()
    }
/*
    fn full_pathname<'a>(&self, db: &'a str) -> Result<Cow<'a, str>, std::io::Error> {
        let path = Path::new(&db);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        let path = normalize_path(&path);
        Ok(path
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(
                    ErrorKind::Other,
                    "cannot convert canonicalized path to string",
                )
            })?
            .to_string()
            .into())
    }

    fn random(&self, buffer: &mut [i8]) {
        rand::Rng::fill(&mut rand::thread_rng(), buffer);
    }

    fn sleep(&self, duration: std::time::Duration) -> std::time::Duration {
        std::thread::sleep(duration);

        // Well, this function is only supposed to sleep at least `n_micro`μs, but there are
        // tests that expect the return to match exactly `n_micro`. As those tests are flaky as
        // a result, we are cheating here.
        duration
    }
*/    
}

impl DatabaseHandle for Connection {
    type WalIndex = WalConnection;

    /*
    fn size(&self) -> Result<u64, std::io::Error> {
        self.file.metadata().map(|m| m.len())
    }

    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> Result<(), std::io::Error> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(buf)
    }

    fn write_all_at(&mut self, buf: &[u8], offset: u64) -> Result<(), std::io::Error> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(buf)?;
        Ok(())
    }

    fn sync(&mut self, data_only: bool) -> Result<(), std::io::Error> {
        if data_only {
            self.file.sync_data()
        } else {
            self.file.sync_all()
        }
    }

    fn set_len(&mut self, len: u64) -> Result<(), std::io::Error> {
        self.file.set_len(len)
    }
    */

    fn lock(&mut self, to: LockKind) -> Result<bool, std::io::Error> {
        let lock = match &mut self.lock {
            Some(lock) => lock,
            None => self.lock.get_or_insert(Lock::from_file(&self.file)?),
        };

        // Return false if exclusive was requested and only pending was acquired.
        Ok(lock.lock(to) && lock.current() == to)
    }

    fn reserved(&mut self) -> Result<bool, std::io::Error> {
        let lock = match &mut self.lock {
            Some(lock) => lock,
            None => self.lock.get_or_insert(Lock::from_file(&self.file)?),
        };

        Ok(lock.reserved())
    }

    fn current_lock(&self) -> Result<LockKind, std::io::Error> {
        Ok(self
            .lock
            .as_ref()
            .map(|l| l.current())
            .unwrap_or(LockKind::None))
    }

    fn moved(&self) -> Result<bool, std::io::Error> {
        let ino = fs::metadata(&self.path).map(|m| m.ino()).unwrap_or(0);
        Ok(ino == 0 || ino != self.file_ino)
    }

    fn wal_index(&self, readonly: bool) -> Result<Self::WalIndex, std::io::Error> {
        let path = self.path.with_extension(format!(
            "{}-shm",
            self.path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.split_once('-').map(|(f, _)| f).unwrap_or(ext))
                .unwrap_or("db")
        ));
        let is_new = !path.exists();

        let mut opts = fs::OpenOptions::new();
        opts.read(true);
        if !readonly {
            opts.write(true).create(true).truncate(false);
        }

        let file = opts.open(&path)?;
        let mut file_lock = FileLock::new(file);
        if !readonly && file_lock.exclusive() {
            // If it is the first connection to open the database, truncate the index.
            let new_file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .map_err(|err| err)?;

            let new_lock = FileLock::new(new_file);

            if is_new {
                let mode = permissions(&self.path)?;
                let perm = Permissions::from_mode(mode);
                // Match permissions of main db file, but don't downgrade to readonly.
                if !perm.readonly() {
                    fs::set_permissions(&path, perm)?;
                }
            }

            // Transition previous lock to shared before getting a shared on the new file
            // descriptor to make sure that there isn't any other concurrent process/thread getting
            // an exclusive lock during the transition.
            assert!(file_lock.shared());
            assert!(new_lock.shared());

            file_lock = new_lock;
        } else {
            file_lock.wait_shared();
        }

        Ok(WalConnection {
            path,
            file_lock,
            wal_lock: RangeLock::new(self.file_ino),
            readonly,
        })
    }
}

impl WalIndex for WalConnection {
    fn map(&mut self, region: u32) -> Result<[u8; 32768], std::io::Error> {
        let mut data = [0u8; 32768];
        self.pull(region, &mut data)?;
        Ok(data)
    }

    fn lock(
        &mut self,
        locks: std::ops::Range<u8>,
        lock: LockKind,
    ) -> Result<bool, std::io::Error> {
        self.wal_lock.lock(locks, lock)
    }

    fn delete(self) -> Result<(), std::io::Error> {
        fs::remove_file(&self.path)
    }

    fn pull(&mut self, region: u32, data: &mut [u8; 32768]) -> Result<(), std::io::Error> {
        let current_size = self.file_lock.file().metadata()?.size();
        let min_size = (region as u64 + 1) * 32768;
        if !self.readonly && current_size < min_size {
            self.file_lock.file().set_len(min_size)?;
        }

        self.file_lock
            .file()
            .seek(SeekFrom::Start(region as u64 * 32768))?;
        match self.file_lock.file().read_exact(data) {
            Ok(()) => Ok(()),
            Err(err) if self.readonly && err.kind() == ErrorKind::UnexpectedEof => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn push(&mut self, region: u32, data: &[u8; 32768]) -> Result<(), std::io::Error> {
        let current_size = self.file_lock.file().metadata()?.size();
        let min_size = (region as u64 + 1) * 32768;
        if current_size < min_size {
            self.file_lock.file().set_len(min_size)?;
        }

        self.file_lock
            .file()
            .seek(SeekFrom::Start(region as u64 * 32768))?;
        self.file_lock.file().write_all(data)?;
        self.file_lock.file().sync_all()?;

        Ok(())
    }
}

// Source: https://github.com/rust-lang/cargo/blob/7a3b56b4860c0e58dab815549a93198a1c335b64/crates/cargo-util/src/paths.rs#L81
fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

fn permissions(path: &Path) -> io::Result<u32> {
    let path = path.with_extension(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.split_once('-').map(|(f, _)| f).unwrap_or(ext))
            .unwrap_or("db"),
    );
    Ok(fs::metadata(&path)?.permissions().mode())
}
