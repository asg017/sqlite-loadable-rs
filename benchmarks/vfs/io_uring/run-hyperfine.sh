#!/bin/sh

which hyperfine || cargo install --locked hyperfine
cargo build --examples

mkdir -p db md

# echo "journal"

# # mem_vfs io_uring_vfs unix_vfs (latter 2 with default rollback journal)
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_1" "./target/debug/examples/test_1 db/test_1.db" "./target/debug/examples/test_1 db/test_1.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_2" "./target/debug/examples/test_2 db/test_2.db" "./target/debug/examples/test_2 db/test_2.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_3" "./target/debug/examples/test_3 db/test_3.db" "./target/debug/examples/test_3 db/test_3.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_4" "./target/debug/examples/test_4 db/test_4.db" "./target/debug/examples/test_4 db/test_4.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_5" "./target/debug/examples/test_5 db/test_5.db" "./target/debug/examples/test_5 db/test_5.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_6" "./target/debug/examples/test_6 db/test_6.db" "./target/debug/examples/test_6 db/test_6.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_7" "./target/debug/examples/test_7 db/test_7.db" "./target/debug/examples/test_7 db/test_7.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_8" "./target/debug/examples/test_8 db/test_8.db" "./target/debug/examples/test_8 db/test_8.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_9" "./target/debug/examples/test_9 db/test_9.db" "./target/debug/examples/test_9 db/test_9.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_10" "./target/debug/examples/test_10 db/test_10.db" "./target/debug/examples/test_10 db/test_10.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_11" "./target/debug/examples/test_11 db/test_11.db" "./target/debug/examples/test_11 db/test_11.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_12" "./target/debug/examples/test_12 db/test_12.db" "./target/debug/examples/test_12 db/test_12.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_13" "./target/debug/examples/test_13 db/test_13.db" "./target/debug/examples/test_13 db/test_13.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_14" "./target/debug/examples/test_14 db/test_14.db" "./target/debug/examples/test_14 db/test_14.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_15" "./target/debug/examples/test_15 db/test_15.db" "./target/debug/examples/test_15 db/test_15.ring.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_16" "./target/debug/examples/test_16 db/test_16.db" "./target/debug/examples/test_16 db/test_16.ring.db"

# echo "wal"

# # mem_vfs io_uring_vfs+wal unix_vfs+wal
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_1" "./target/debug/examples/test_1 db/test_1.wal.db" "./target/debug/examples/test_1 db/test_1.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_2" "./target/debug/examples/test_2 db/test_2.wal.db" "./target/debug/examples/test_2 db/test_2.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_3" "./target/debug/examples/test_3 db/test_3.wal.db" "./target/debug/examples/test_3 db/test_3.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_4" "./target/debug/examples/test_4 db/test_4.wal.db" "./target/debug/examples/test_4 db/test_4.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_5" "./target/debug/examples/test_5 db/test_5.wal.db" "./target/debug/examples/test_5 db/test_5.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_6" "./target/debug/examples/test_6 db/test_6.wal.db" "./target/debug/examples/test_6 db/test_6.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_7" "./target/debug/examples/test_7 db/test_7.wal.db" "./target/debug/examples/test_7 db/test_7.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_8" "./target/debug/examples/test_8 db/test_8.wal.db" "./target/debug/examples/test_8 db/test_8.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_9" "./target/debug/examples/test_9 db/test_9.wal.db" "./target/debug/examples/test_9 db/test_9.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_10" "./target/debug/examples/test_10 db/test_10.wal.db" "./target/debug/examples/test_10 db/test_10.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_11" "./target/debug/examples/test_11 db/test_11.wal.db" "./target/debug/examples/test_11 db/test_11.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_12" "./target/debug/examples/test_12 db/test_12.wal.db" "./target/debug/examples/test_12 db/test_12.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_13" "./target/debug/examples/test_13 db/test_13.wal.db" "./target/debug/examples/test_13 db/test_13.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_14" "./target/debug/examples/test_14 db/test_14.wal.db" "./target/debug/examples/test_14 db/test_14.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_15" "./target/debug/examples/test_15 db/test_15.wal.db" "./target/debug/examples/test_15 db/test_15.ring.wal.db"
# hyperfine --show-output --warmup 3 "./target/debug/examples/test_16" "./target/debug/examples/test_16 db/test_16.wal.db" "./target/debug/examples/test_16 db/test_16.ring.db"

echo "everything together"

# mem_vfs io_uring_vfs+rollback unix_vfs+rollback io_uring_vfs+wal unix_vfs+wal
hyperfine -N --export-markdown md/test_1.md --show-output --warmup 3 "./target/debug/examples/test_1" "./target/debug/examples/test_1 db/test_1.db" "./target/debug/examples/test_1 db/test_1.ring.db" "./target/debug/examples/test_1 db/test_1.ring.wal.db" "./target/debug/examples/test_1 db/test_1.wal.db"
hyperfine -N --export-markdown md/test_2.md --show-output --warmup 3 "./target/debug/examples/test_2" "./target/debug/examples/test_2 db/test_2.db" "./target/debug/examples/test_2 db/test_2.ring.db" "./target/debug/examples/test_2 db/test_2.ring.wal.db" "./target/debug/examples/test_2 db/test_2.wal.db"
hyperfine -N --export-markdown md/test_3.md --show-output --warmup 3 "./target/debug/examples/test_3" "./target/debug/examples/test_3 db/test_3.db" "./target/debug/examples/test_3 db/test_3.ring.db" "./target/debug/examples/test_3 db/test_3.ring.wal.db" "./target/debug/examples/test_3 db/test_3.wal.db"
hyperfine -N --export-markdown md/test_4.md --show-output --warmup 3 "./target/debug/examples/test_4" "./target/debug/examples/test_4 db/test_4.db" "./target/debug/examples/test_4 db/test_4.ring.db" "./target/debug/examples/test_4 db/test_4.ring.wal.db" "./target/debug/examples/test_4 db/test_4.wal.db"
hyperfine -N --export-markdown md/test_5.md --show-output --warmup 3 "./target/debug/examples/test_5" "./target/debug/examples/test_5 db/test_5.db" "./target/debug/examples/test_5 db/test_5.ring.db" "./target/debug/examples/test_5 db/test_5.ring.wal.db" "./target/debug/examples/test_5 db/test_5.wal.db"
hyperfine -N --export-markdown md/test_6.md --show-output --warmup 3 "./target/debug/examples/test_6" "./target/debug/examples/test_6 db/test_6.db" "./target/debug/examples/test_6 db/test_6.ring.db" "./target/debug/examples/test_6 db/test_6.ring.wal.db" "./target/debug/examples/test_6 db/test_6.wal.db"
hyperfine -N --export-markdown md/test_7.md --show-output --warmup 3 "./target/debug/examples/test_7" "./target/debug/examples/test_7 db/test_7.db" "./target/debug/examples/test_7 db/test_7.ring.db" "./target/debug/examples/test_7 db/test_7.ring.wal.db" "./target/debug/examples/test_7 db/test_7.wal.db"
hyperfine -N --export-markdown md/test_8.md --show-output --warmup 3 "./target/debug/examples/test_8" "./target/debug/examples/test_8 db/test_8.db" "./target/debug/examples/test_8 db/test_8.ring.db" "./target/debug/examples/test_8 db/test_8.ring.wal.db" "./target/debug/examples/test_8 db/test_8.wal.db"
hyperfine -N --export-markdown md/test_9.md --show-output --warmup 3 "./target/debug/examples/test_9" "./target/debug/examples/test_9 db/test_9.db" "./target/debug/examples/test_9 db/test_9.ring.db" "./target/debug/examples/test_9 db/test_9.ring.wal.db" "./target/debug/examples/test_9 db/test_9.wal.db"
hyperfine -N --export-markdown md/test_10.md --show-output --warmup 3 "./target/debug/examples/test_10" "./target/debug/examples/test_10 db/test_10.db" "./target/debug/examples/test_10 db/test_10.ring.db" "./target/debug/examples/test_10 db/test_10.ring.wal.db" "./target/debug/examples/test_10 db/test_10.wal.db"
hyperfine -N --export-markdown md/test_11.md --show-output --warmup 3 "./target/debug/examples/test_11" "./target/debug/examples/test_11 db/test_11.db" "./target/debug/examples/test_11 db/test_11.ring.db" "./target/debug/examples/test_11 db/test_11.ring.wal.db" "./target/debug/examples/test_11 db/test_11.wal.db"
hyperfine -N --export-markdown md/test_12.md --show-output --warmup 3 "./target/debug/examples/test_12" "./target/debug/examples/test_12 db/test_12.db" "./target/debug/examples/test_12 db/test_12.ring.db" "./target/debug/examples/test_12 db/test_12.ring.wal.db" "./target/debug/examples/test_12 db/test_12.wal.db"
hyperfine -N --export-markdown md/test_13.md --show-output --warmup 3 "./target/debug/examples/test_13" "./target/debug/examples/test_13 db/test_13.db" "./target/debug/examples/test_13 db/test_13.ring.db" "./target/debug/examples/test_13 db/test_13.ring.wal.db" "./target/debug/examples/test_13 db/test_13.wal.db"
hyperfine -N --export-markdown md/test_14.md --show-output --warmup 3 "./target/debug/examples/test_14" "./target/debug/examples/test_14 db/test_14.db" "./target/debug/examples/test_14 db/test_14.ring.db" "./target/debug/examples/test_14 db/test_14.ring.wal.db" "./target/debug/examples/test_14 db/test_14.wal.db"
hyperfine -N --export-markdown md/test_15.md --show-output --warmup 3 "./target/debug/examples/test_15" "./target/debug/examples/test_15 db/test_15.db" "./target/debug/examples/test_15 db/test_15.ring.db" "./target/debug/examples/test_15 db/test_15.ring.wal.db" "./target/debug/examples/test_15 db/test_15.wal.db"
hyperfine -N --export-markdown md/test_16.md --show-output --warmup 3 "./target/debug/examples/test_16" "./target/debug/examples/test_16 db/test_16.db" "./target/debug/examples/test_16 db/test_16.ring.db" "./target/debug/examples/test_16 db/test_16.ring.wal.db" "./target/debug/examples/test_16 db/test_16.wal.db"

# for i in `seq 16`; do
#   echo "hyperfine --export-markdown md/test_${i}.md --show-output --warmup 3 \"./target/debug/examples/test_$i\" \"./target/debug/examples/test_$i db/test_${i}.db\" \"./target/debug/examples/test_$i db/test_${i}.ring.db\" \"./target/debug/examples/test_$i db/test_${i}.ring.wal.db\" \"./target/debug/examples/test_$i db/test_${i}.wal.db\""; \
# done

