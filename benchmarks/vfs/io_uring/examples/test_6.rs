use std::env;
use rand::Rng;

include!("conn.in");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args.len() > 0);
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;
    for _ in 0..500000 {
        let value1: i32 = rng.gen();
        let value2: i32 = rng.gen();

        tx.execute("INSERT INTO t6 (a, b) VALUES (?, ?)",
                            (value1, value2))?;
    }
    tx.commit()?;

    conn.execute("CREATE INDEX i6a ON t6(a)", ())?;
    conn.execute("CREATE INDEX i6b ON t6(b)", ())?;
    
    Ok(())
}
