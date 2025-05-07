use rand::Rng;
use std::env;

include!("../include/conn.in.rs");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args)?;
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;
    for _ in 0..25000 {
        let value1: i32 = rng.gen();
        let value2: i32 = rng.gen();

        tx.execute("INSERT INTO t6 (a, b) VALUES (?, ?)", (value1, value2))?;
    }
    tx.commit()?;

    // fails if file is already indexed, TODO fix
    conn.execute("CREATE INDEX IF NOT EXISTS i6a ON t6(a)", ())?;
    conn.execute("CREATE INDEX IF NOT EXISTS i6b ON t6(b)", ())?;

    Ok(())
}
