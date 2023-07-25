/// rust bindgen for some reason is defining many SQLite constants
/// as u32, which can't safely be casted into i32. So, here we
/// hardcode some of those codes to avoid unwrapping

/// https://www.sqlite.org/rescode.html#constraint
pub const SQLITE_CONSTRAINT: i32 = 19;

/// https://www.sqlite.org/rescode.html#error
pub const SQLITE_ERROR: i32 = 1;

/// https://www.sqlite.org/rescode.html#ok
pub const SQLITE_OKAY: i32 = 0;

/// https://www.sqlite.org/rescode.html#internal
pub const SQLITE_INTERNAL: i32 = 2;

/// https://www.sqlite.org/rescode.html#done
pub const SQLITE_DONE: i32 = 101;
