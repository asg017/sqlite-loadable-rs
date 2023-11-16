#!/bin/sh

rm -f *db *journal

which hyperfine || cargo install --locked hyperfine
cargo build --examples

mkdir -p db

# mem_vfs io_uring_vfs file_db_vfs
hyperfine --show-output --warmup 3 "./target/debug/examples/test_1" "./target/debug/examples/test_1 db/test_1.db" "./target/debug/examples/test_1 db/test_1.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_2" "./target/debug/examples/test_2 db/test_2.db" "./target/debug/examples/test_2 db/test_2.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_3" "./target/debug/examples/test_3 db/test_3.db" "./target/debug/examples/test_3 db/test_3.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_4" "./target/debug/examples/test_4 db/test_4.db" "./target/debug/examples/test_4 db/test_4.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_5" "./target/debug/examples/test_5 db/test_5.db" "./target/debug/examples/test_5 db/test_5.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_6" "./target/debug/examples/test_6 db/test_6.db" "./target/debug/examples/test_6 db/test_6.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_7" "./target/debug/examples/test_7 db/test_7.db" "./target/debug/examples/test_7 db/test_7.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_8" "./target/debug/examples/test_8 db/test_8.db" "./target/debug/examples/test_8 db/test_8.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_9" "./target/debug/examples/test_9 db/test_9.db" "./target/debug/examples/test_9 db/test_9.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_10" "./target/debug/examples/test_10 db/test_10.db" "./target/debug/examples/test_10 db/test_10.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_11" "./target/debug/examples/test_11 db/test_11.db" "./target/debug/examples/test_11 db/test_11.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_12" "./target/debug/examples/test_12 db/test_12.db" "./target/debug/examples/test_12 db/test_12.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_13" "./target/debug/examples/test_13 db/test_13.db" "./target/debug/examples/test_13 db/test_13.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_14" "./target/debug/examples/test_14 db/test_14.db" "./target/debug/examples/test_14 db/test_14.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_15" "./target/debug/examples/test_15 db/test_15.db" "./target/debug/examples/test_15 db/test_15.ring.db"
hyperfine --show-output --warmup 3 "./target/debug/examples/test_16" "./target/debug/examples/test_16 db/test_16.db" "./target/debug/examples/test_16 db/test_16.ring.db"

# for i in `seq 16`; do
#   echo "hyperfine --show-output --warmup 3 \"./target/debug/examples/test_$i\" \"./target/debug/examples/test_$i db/test_${i}.db\" \"./target/debug/examples/test_$i db/test_${i}.ring.db\""; \
# done

