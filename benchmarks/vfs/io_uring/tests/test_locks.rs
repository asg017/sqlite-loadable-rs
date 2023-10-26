#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    
    use _iouringvfs::lock::*;

    fn test_file(name: &str) -> PathBuf {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(name)
            .with_extension("txt");
        fs::write(&path, "").unwrap();
        path
    }

    #[test]
    fn test_lock_order() {
        assert!(LockKind::None < LockKind::Shared);
        assert!(LockKind::Shared < LockKind::Reserved);
        assert!(LockKind::Reserved < LockKind::Pending);
        assert!(LockKind::Pending < LockKind::Exclusive);
    }

    #[test]
    fn test_none() {
        let path = test_file(".test_none");
        let lock = Lock::new(&path).unwrap();
        assert_eq!(lock.current(), LockKind::None);
    }
 
    #[test]
    fn test_shared() {
        let path = test_file(".test_shared");
        let mut lock = Lock::new(&path).unwrap();
        assert!(lock.lock(LockKind::Shared));
        assert_eq!(lock.current(), LockKind::Shared);
    }

    #[test]
    fn test_reserved() {
        let path = test_file(".test_reserved");
        let mut lock = Lock::new(&path).unwrap();
        assert!(lock.lock(LockKind::Shared));
        assert!(lock.lock(LockKind::Reserved));
        assert_eq!(lock.current(), LockKind::Reserved);
    }

    #[test]
    fn test_exclusive() {
        let path = test_file(".test_exclusive");
        let mut lock = Lock::new(&path).unwrap();
        assert!(lock.lock(LockKind::Shared));
        assert!(lock.lock(LockKind::Exclusive));
        assert_eq!(lock.current(), LockKind::Exclusive);
    }

    #[test]
    fn test_exclusive_via_reserved() {
        let path = test_file(".test_exclusive_via_reserved");
        let mut lock = Lock::new(&path).unwrap();
        assert!(lock.lock(LockKind::Shared));
        assert!(lock.lock(LockKind::Reserved));
        assert!(lock.lock(LockKind::Exclusive));
        assert_eq!(lock.current(), LockKind::Exclusive);
    }

    #[test]
    #[should_panic(
        expected = "cannot transition from unlocked to anything higher than shared (tried: Reserved)"
    )]
    fn test_none_to_reserved_panic() {
        let path = test_file(".test_none_to_reserved_panic");
        let mut lock = Lock::new(&path).unwrap();
        lock.lock(LockKind::Reserved);
    }

    #[test]
    #[should_panic(
        expected = "cannot transition from unlocked to anything higher than shared (tried: Exclusive)"
    )]
    fn test_none_to_exclusive_panic() {
        let path = test_file(".test_none_to_exclusive_panic");
        let mut lock = Lock::new(&path).unwrap();
        lock.lock(LockKind::Exclusive);
    }

    #[test]
    #[should_panic(expected = "cannot explicitly request pending lock (request explicit lock instead)")]
    fn test_shared_to_pending_panic() {
        let path = test_file(".test_shared_to_pending_panic");
        let mut lock = Lock::new(&path).unwrap();
        assert!(lock.lock(LockKind::Shared));
        lock.lock(LockKind::Pending);
    }

    #[test]
    #[should_panic(expected = "cannot explicitly request pending lock (request explicit lock instead)")]
    fn test_reserved_to_pending_panic() {
        let path = test_file(".test_reserved_to_pending_panic");
        let mut lock = Lock::new(&path).unwrap();
        assert!(lock.lock(LockKind::Shared));
        assert!(lock.lock(LockKind::Reserved));
        lock.lock(LockKind::Pending);
    }

    #[test]
    fn test_reserved_once() {
        let path = test_file(".reserved_once");
        let mut lock1 = Lock::new(&path).unwrap();
        assert!(lock1.lock(LockKind::Shared));

        let mut lock2 = Lock::new(&path).unwrap();
        assert!(lock2.lock(LockKind::Shared));

        assert!(lock1.lock(LockKind::Reserved));
        assert!(!lock2.lock(LockKind::Reserved));

        assert!(lock1.lock(LockKind::Shared));
        assert!(lock2.lock(LockKind::Reserved));
    }

    #[test]
    fn test_shared_while_reserved() {
        let path = test_file(".shared_while_reserved");
        let mut lock1 = Lock::new(&path).unwrap();
        assert!(lock1.lock(LockKind::Shared));
        assert!(lock1.lock(LockKind::Reserved));

        let mut lock2 = Lock::new(&path).unwrap();
        assert!(lock2.lock(LockKind::Shared));
    }

    #[test]
    fn test_pending() {
        let path = test_file(".test_pending");
        let mut lock1 = Lock::new(&path).unwrap();
        assert!(lock1.lock(LockKind::Shared));

        let mut lock2 = Lock::new(&path).unwrap();
        assert!(lock2.lock(LockKind::Shared));
        assert!(lock2.lock(LockKind::Exclusive));
        assert_eq!(lock2.current(), LockKind::Pending);
    }

    #[test]
    fn test_pending_once() {
        let path = test_file(".test_pending_once");
        let mut lock1 = Lock::new(&path).unwrap();
        assert!(lock1.lock(LockKind::Shared));

        let mut lock2 = Lock::new(&path).unwrap();
        assert!(lock2.lock(LockKind::Shared));
        assert!(lock2.lock(LockKind::Exclusive));

        assert!(!lock1.lock(LockKind::Exclusive));

        assert_eq!(lock1.current(), LockKind::Shared);
        assert_eq!(lock2.current(), LockKind::Pending);
    }

    #[test]
    fn test_pending_to_exclusive() {
        let path = test_file(".test_pending_to_exclusive");
        let mut lock1 = Lock::new(&path).unwrap();
        assert!(lock1.lock(LockKind::Shared));

        let mut lock2 = Lock::new(&path).unwrap();
        assert!(lock2.lock(LockKind::Shared));
        assert!(lock2.lock(LockKind::Exclusive));

        assert!(lock1.lock(LockKind::None));
        assert!(lock2.lock(LockKind::Exclusive));

        assert_eq!(lock1.current(), LockKind::None);
        assert_eq!(lock2.current(), LockKind::Exclusive);
    }
}
