use std::env;
use rand::Rng;

include!("conn.in");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args)?;
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;
    for _ in 0..5000 {
        let value: i32 = rng.gen();

        tx.execute("INSERT INTO t4 (a, b, c) VALUES (?, ?, ?)",
                            (value, value, format!("Value {}", value).as_str()))?;
    }
    tx.commit()?;

    let tx2 = conn.transaction()?;
    tx2.execute("DELETE FROM t4",())?;
    for _ in 0..5000 {
        let value: i32 = rng.gen();

        tx2.execute("INSERT INTO t4 (a, b, c) VALUES (?, ?, ?)",
                            (value, value, format!("Value {}", value).as_str()))?;
    }

    tx2.commit()?;
    
    Ok(())
}
