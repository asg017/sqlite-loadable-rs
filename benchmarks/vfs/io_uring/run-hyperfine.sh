#!/bin/sh

rm -f *db *journal

which hyperfine || cargo install --locked hyperfine
cargo build --examples

# mem_vfs io_uring_vfs file_db_vfs
hyperfine --show-output --warmup 1 "./target/debug/examples/test_1" "./target/debug/examples/test_1 test_1.db" "./target/debug/examples/test_1 test_1_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_2" "./target/debug/examples/test_2 test_2.db" "./target/debug/examples/test_2 test_2_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_3" "./target/debug/examples/test_3 test_3.db" "./target/debug/examples/test_3 test_3_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_4" "./target/debug/examples/test_4 test_4.db" "./target/debug/examples/test_4 test_4_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_5" "./target/debug/examples/test_5 test_5.db" "./target/debug/examples/test_5 test_5_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_6" "./target/debug/examples/test_6 test_6.db" "./target/debug/examples/test_6 test_6_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_7" "./target/debug/examples/test_7 test_7.db" "./target/debug/examples/test_7 test_7_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_8" "./target/debug/examples/test_8 test_8.db" "./target/debug/examples/test_8 test_8_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_9" "./target/debug/examples/test_9 test_9.db" "./target/debug/examples/test_9 test_9_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_10" "./target/debug/examples/test_10 test_10.db" "./target/debug/examples/test_10 test_10_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_11" "./target/debug/examples/test_11 test_11.db" "./target/debug/examples/test_11 test_11_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_12" "./target/debug/examples/test_12 test_12.db" "./target/debug/examples/test_12 test_12_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_13" "./target/debug/examples/test_13 test_13.db" "./target/debug/examples/test_13 test_13_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_14" "./target/debug/examples/test_14 test_14.db" "./target/debug/examples/test_14 test_14_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_15" "./target/debug/examples/test_15 test_15.db" "./target/debug/examples/test_15 test_15_ring.db"
hyperfine --show-output --warmup 1 "./target/debug/examples/test_16" "./target/debug/examples/test_16 test_16.db" "./target/debug/examples/test_16 test_16_ring.db"

# for i in `seq 16`; do
#   echo "hyperfine --show-output --warmup 1 \"./target/debug/examples/test_$i\" \"./target/debug/examples/test_$i test_$i.db\" \"./target/debug/examples/test_$i test_$i_ring.db\""; \
# done

