use bitflags::bitflags;

use sqlite3ext_sys::{
    SQLITE_DETERMINISTIC, SQLITE_DIRECTONLY, SQLITE_INNOCUOUS, SQLITE_SUBTYPE, SQLITE_UTF16,
    SQLITE_UTF16BE, SQLITE_UTF16LE, SQLITE_UTF8,
};

bitflags! {
    /// Represents the possible flag values that can be passed into sqlite3_create_function_v2
    /// or sqlite3_create_window_function, as the 4th "eTextRep" parameter.
    /// Includes both the encoding options (utf8, utf16, etc.) and function-level parameters
    /// (deterministion, innocuous, etc.).
    pub struct FunctionFlags: i32 {
        const UTF8 = SQLITE_UTF8 as i32;
        const UTF16LE = SQLITE_UTF16LE as i32;
        const UTF16BE = SQLITE_UTF16BE as i32;
        const UTF16 = SQLITE_UTF16 as i32;

        /// "... to signal that the function will always return the same result given the same
        /// inputs within a single SQL statement."
        /// <https://www.sqlite.org/c3ref/create_function.html#:~:text=ORed%20with%20SQLITE_DETERMINISTIC>
        const DETERMINISTIC = SQLITE_DETERMINISTIC as i32;
        const DIRECTONLY = SQLITE_DIRECTONLY as i32;
        const SUBTYPE = SQLITE_SUBTYPE as i32;
        const INNOCUOUS = SQLITE_INNOCUOUS as i32;
    }
}
