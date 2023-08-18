use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, define_scalar_function, Result};

pub fn hello(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let name = api::value_text(values.get(0).expect("1st argument as name"))?;

    api::result_text(context, format!("hello, {}!", name))?;
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_hello_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "hello", 1, hello, flags)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rusqlite::{ffi::sqlite3_auto_extension, Connection};

    #[test]
    fn test_rusqlite_auto_extension() {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_hello_init as *const ())));
        }

        let conn = Connection::open_in_memory().unwrap();

        let result: String = conn
            .query_row("select hello(?)", ["alex"], |x| x.get(0))
            .unwrap();

        assert_eq!(result, "hello, alex!");
    }
}
