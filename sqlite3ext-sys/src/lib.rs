#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        assert_eq!(SQLITE_VERSION_NUMBER, 3039004);
        assert_eq!(unsafe { sqlite3_libversion_number() }, 3039004);
    }
}
