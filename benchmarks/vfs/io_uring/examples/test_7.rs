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

        tx.execute("INSERT INTO t7 (a, b) VALUES (?, ?)", (value1, value2))?;
    }
    tx.commit()?;

    for i in 0..5000 {
        let lower_bound = i * 100;
        let upper_bound = (i + 1) + 100;

        let _ = conn.prepare("SELECT count(*), avg(b) FROM t7 WHERE b >= ?1 AND b < ?2")?.
            query([lower_bound, upper_bound])?;
    }
    Ok(())
}
