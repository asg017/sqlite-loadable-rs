#!/bin/sh

which hyperfine || cargo install --locked hyperfine
cargo build --examples

# mem_vfs io_uring_vfs file_db_vfs
hyperfine --show-output --warmup 1 "./target/debug/examples/test_1" "./target/debug/examples/test_1 test_1.db" "./target/debug/examples/test_1 test_1.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_2" "./target/debug/examples/test_2 test_2.db" "./target/debug/examples/test_2 test_2.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_3" "./target/debug/examples/test_3 test_3.db" "./target/debug/examples/test_3 test_3.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_4" "./target/debug/examples/test_4 test_4.db" "./target/debug/examples/test_4 test_4.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_5" "./target/debug/examples/test_5 test_5.db" "./target/debug/examples/test_5 test_5.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_6" "./target/debug/examples/test_6 test_6.db" "./target/debug/examples/test_6 test_6.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_7" "./target/debug/examples/test_7 test_7.db" "./target/debug/examples/test_7 test_7.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_8" "./target/debug/examples/test_8 test_8.db" "./target/debug/examples/test_8 test_8.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_9" "./target/debug/examples/test_9 test_9.db" "./target/debug/examples/test_9 test_9.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_10" "./target/debug/examples/test_10 test_10.db" "./target/debug/examples/test_10 test_10.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_11" "./target/debug/examples/test_11 test_11.db" "./target/debug/examples/test_11 test_11.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_12" "./target/debug/examples/test_12 test_12.db" "./target/debug/examples/test_12 test_12.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_13" "./target/debug/examples/test_13 test_13.db" "./target/debug/examples/test_13 test_13.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_14" "./target/debug/examples/test_14 test_14.db" "./target/debug/examples/test_14 test_14.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_15" "./target/debug/examples/test_15 test_15.db" "./target/debug/examples/test_15 test_15.ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_16" "./target/debug/examples/test_16 test_16.db" "./target/debug/examples/test_16 test_16.ring.db"

# for i in {16,15,14,13,12,11,10,9}; do; \
#   echo "hyperfine --show-output --warmup 1 \"./target/debug/examples/test_$i\" \"./target/debug/examples/test_$i test_$i.db\" \"./target/debug/examples/test_$i test_$i.ring.db\""; \
# done
