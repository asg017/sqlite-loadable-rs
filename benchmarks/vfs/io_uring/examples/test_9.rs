use std::env;
use rand::Rng;

include!("conn.in");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args);
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;
    for _ in 0..5000 {
        let value: i32 = rng.gen();

        tx.execute("INSERT INTO t9 (a, b) VALUES (?, ?)",
                            (value, value))?;
    }
    tx.commit()?;

    let tx2 = conn.transaction()?;
    for i in 0..5000 {
        let r: i32 = rng.gen();
        let upper_bound = i + 1;

        let _ = tx2.execute("UPDATE t9 SET b=?1 WHERE a = ?2", (r, upper_bound));
    }
    tx2.commit()?;
    
    Ok(())
}
