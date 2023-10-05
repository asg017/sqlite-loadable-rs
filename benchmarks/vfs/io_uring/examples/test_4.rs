use std::env;
use rand::Rng;
use rand::thread_rng;

include!("conn.in");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args.len() > 0);
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;
    for _ in 0..25000 {
        let value1: i32 = rng.gen();
        let value2: i32 = rng.gen();
        let value3: String = format!("Value{}", thread_rng().gen_range(0..25000));

        tx.execute("INSERT INTO t4 (a, b, c) VALUES (?, ?, ?)",
                            (value1, value2, value3))?;
    }
    tx.commit()?;
    
    let tx2 = conn.transaction()?;
    for i in 0..100 {
        let lower_bound = i * 100;
        let upper_bound = (i + 1) * 1000;

        let _ = tx2.execute("SELECT count(*), avg(b) FROM t4 WHERE b >= ?1 AND b < ?2", (lower_bound, upper_bound));
    }
    tx2.commit()?;
    Ok(())
}
