use std::env;
use rand::Rng;
use rand::thread_rng;

include!("../include/conn.in.rs");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let conn = create_test_database(args)?;

    let mut stmt = conn.prepare_cached("INSERT INTO t1 (a, b) VALUES (?, ?)")?;

    for _ in 0..1000 {
        let value1: i32 = thread_rng().gen_range(0..1000);
        let value2: String = format!("Value {}", thread_rng().gen_range(0..1000));

        stmt.execute((value1, value2))?;
    }
    Ok(())
}
