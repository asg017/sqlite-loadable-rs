use std::env;
use rand::Rng;
use rand::thread_rng;

include!("conn.in");

fn main() -> rusqlite::Result<()> {
    let args: Vec<String> = env::args().collect();

    let conn = create_test_database(args.len() > 0);
    let rng = rand::thread_rng();

    for _ in 0..1000 {
        let value1: i32 = thread_rng().gen_range(0..1000);
        let value2: String = format!("Value{}", thread_rng().gen_range(0..1000));

        conn.execute("INSERT INTO t1 (a, b) VALUES (?, ?)",
            (value1, value2))
            .expect("Failed to insert data");
    }
    Ok(())
}
