use std::env;
use rand::Rng;
use rand::thread_rng;

include!("conn.in");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args);
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;

    for _ in 0..5000 {
        let value1: i32 = rng.gen();
        let value2: i32 = rng.gen();
        let value3: String = format!("Value {}", thread_rng().gen_range(0..25000));

        tx.execute("INSERT INTO t2 (a, b, c) VALUES (?, ?, ?)",
                            (value1, value2, value3))?;
    }

    tx.commit()?;

    conn.execute("DELETE FROM t2 WHERE c LIKE '%50%'", ())?;

    Ok(())
}
