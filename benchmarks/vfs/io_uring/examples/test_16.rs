use rand::Rng;
use std::env;

include!("../include/conn.in.rs");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args)?;
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;
    for _ in 0..5000 {
        let value: i32 = rng.gen();

        tx.execute(
            "INSERT INTO t4 (a, b, c) VALUES (?, ?, ?)",
            (value, value, format!("Value {}", value).as_str()),
        )?;
        tx.execute(
            "INSERT INTO t5 (a, b, c) VALUES (?, ?, ?)",
            (value, value, format!("Value {}", value).as_str()),
        )?;
    }
    tx.commit()?;

    conn.execute("DELETE FROM t4", ())?;

    Ok(())
}
