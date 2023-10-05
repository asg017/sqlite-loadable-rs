use std::env;
use rand::Rng;
use rand::thread_rng;

include!("conn.in");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut conn = create_test_database(args.len() > 0);
    let mut rng = rand::thread_rng();

    let tx = conn.transaction().expect("Failed to start tx");

    for _ in 0..25000 {
        let value1: i32 = rng.gen(); // Generate a random i32 value
        let value2: i32 = rng.gen(); // Generate a random i32 value
        let value3: String = format!("Value {}", thread_rng().gen_range(0..25000));

        tx.execute("INSERT INTO t10 (a, b, c) VALUES (?, ?, ?)",
                            (value1, value2, value3))
            .expect("Failed to insert data");
    }

    tx.commit().expect("Failed to commit transaction");

    conn.execute("DELETE FROM t10 WHERE a>10 AND a <20000", ())?;

    Ok(())
}
