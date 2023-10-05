use std::env;
use rand::Rng;

include!("conn.in");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args);
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;
    for _ in 0..10000 {
        let value1: i32 = rng.gen();
        let value2: i32 = rng.gen();

        tx.execute("INSERT INTO t8 (a, b) VALUES (?, ?)",
                            (value1, value2))?;
    }
    tx.commit()?;

    let tx2 = conn.transaction()?;
    for i in 0..1000 {
        let lower_bound = i * 10;
        let upper_bound = (i + 1) + 10;

        tx2.execute("UPDATE t8 SET b=b*2 WHERE a >= ?1 AND a < ?2", (lower_bound, upper_bound))?;
    }
    tx2.commit()?;

    Ok(())
}
