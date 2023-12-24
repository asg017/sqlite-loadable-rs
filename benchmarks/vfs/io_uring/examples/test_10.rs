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
            "INSERT INTO t10 (a, b, c) VALUES (?, ?, ?)",
            (value, value, format!("Value {}", value).as_str()),
        )?;
    }
    tx.commit()?;

    let tx2 = conn.transaction()?;
    for i in 0..5000 {
        let r: i32 = rng.gen();
        tx2.execute("UPDATE t10 SET c=?1 WHERE a = ?2", (r, i + 1))?;
    }
    tx2.commit()?;

    Ok(())
}
