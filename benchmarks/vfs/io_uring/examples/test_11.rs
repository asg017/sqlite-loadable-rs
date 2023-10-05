use std::env;
use rand::Rng;

include!("conn.in");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args.len() > 0);
    let mut rng = rand::thread_rng();

    let tx = conn.transaction()?;
    for _ in 0..25000 {
        let value: i32 = rng.gen();

        tx.execute("INSERT INTO t4 (a, b, c) VALUES (?, ?, ?)",
                            (value, value, format!("Value {}", value).as_str()))?;
        tx.execute("INSERT INTO t5 (a, b, c) VALUES (?, ?, ?)",
                            (value, value, format!("Value {}", value).as_str()))?;
    }
    tx.commit()?;

    let tx2 = conn.transaction()?;
    tx2.execute("INSERT INTO t4 SELECT b,a,c FROM t5",())?;
    tx2.execute("INSERT INTO t5 SELECT b,a,c FROM t4",())?;
    tx2.commit()?;
    
    Ok(())
}
