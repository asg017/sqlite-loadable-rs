use rand::thread_rng;
use rand::Rng;
use std::env;

include!("../include/conn.in.rs");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args)?;
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;

    for _ in 0..5000 {
        let value1: i32 = rng.gen();
        let value2: i32 = rng.gen();
        let value3: String = format!("Value {}", thread_rng().gen_range(0..25000));

        tx.execute(
            "INSERT INTO t10 (a, b, c) VALUES (?, ?, ?)",
            (value1, value2, value3),
        )?;
    }

    tx.commit()?;

    conn.execute("DELETE FROM t10 WHERE a>10 AND a <20000", ())?;

    Ok(())
}
