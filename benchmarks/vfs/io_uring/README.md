# sqlite3_vfs_io_uring_rs
Performance test: sqlite3 vfs + IO Uring with WAL and rollback journalling

***Warning***: IO Uring is only supported on linux, where this IO Uring has been activated.
IO Uring has been turned off on many distros due to certain security risks.

This project was tested on Docker and VirtualBox. Your mileage will vary.
Also, all of the tests ran on [rusqlite](https://github.com/rusqlite/rusqlite).

## Benchmark speeds with hyperfine

[This script](./run-hyperfine.sh) was written to benchmark and compare, memory vfs as baseline, unix vfs and
the custom IO Uring based vfs, with the default [rollback journalling, and WAL](https://fly.io/blog/sqlite-internals-wal/).

## Tests

[Tests](./examples/) were [derived from this archived sqlite document](https://www.sqlite.org/speed.html),
to show whether adding IO Uring support to a custom vfs will impact sqlite3's performance positively.

16 tests are run, on volatile memory, file storage and file storage via io-uring, where memory storage serves as a baseline.

| Test | Description |
| --- | --- |
| [1](./examples/test_1.rs) | INSERTs |
| [2](./examples/test_2.rs) | INSERTs in a transaction |
| [3](./examples/test_3.rs) | INSERTs into an indexed table |
| [4](./examples/test_4.rs) | SELECTs without an index |
| [5](./examples/test_5.rs) | SELECTs on a string comparison |
| [6](./examples/test_6.rs) | Creating an index |
| [7](./examples/test_7.rs) | SELECTs with an index |
| [8](./examples/test_8.rs) | UPDATEs without an index |
| [9](./examples/test_9.rs) | UPDATEs with an index |
| [10](./examples/test_10.rs) | Text UPDATEs with an index |
| [11](./examples/test_11.rs) | INSERTs from a SELECT |
| [12](./examples/test_12.rs) | DELETE without an index |
| [13](./examples/test_13.rs) | DELETE with an index |
| [14](./examples/test_14.rs) | A big INSERT after a big DELETE |
| [15](./examples/test_15.rs) | A big DELETE followed by many small INSERTs |
| [16](./examples/test_16.rs) | DROP TABLE |

## Run the tests
Run [this script](./run-hyperfine.sh) in a shell
```bash
sh run-hyperfine.sh
```

If you don't have linux running on your machine (yet), use
[the docker script provided here](../../../run-docker.sh).

## Logging

```bash
RUST_LOG=trace cargo test
```

## Results

The numbers here were generated on a noisy machine on Docker.
Your mileage might vary.

Lower "Relative" speed is better.

### Apple M2, Docker

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_1` | 5.3 ± 4.0 | 4.4 | 72.4 | 1.00 |
| `test_1 db/test_1.db` | 1711.4 ± 75.7 | 1631.3 | 1847.3 | 320.26 ± 237.45 |
| `test_1 db/test_1.ring.db` | 1531.8 ± 59.3 | 1488.8 | 1657.8 | 286.65 ± 212.44 |
| `test_1 db/test_1.ring.wal.db` | 1534.6 ± 31.9 | 1498.6 | 1611.6 | 287.18 ± 212.63 |
| `test_1 db/test_1.wal.db` | 230.8 ± 5.7 | 225.2 | 242.4 | 43.19 ± 31.98 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_2` | 98.4 ± 1.9 | 95.5 | 102.6 | 1.00 |
| `test_2 db/test_2.db` | 120.6 ± 1.9 | 115.4 | 125.2 | 1.23 ± 0.03 |
| `test_2 db/test_2.ring.db` | 130.3 ± 1.7 | 127.0 | 133.2 | 1.32 ± 0.03 |
| `test_2 db/test_2.ring.wal.db` | 131.6 ± 5.7 | 128.0 | 154.8 | 1.34 ± 0.06 |
| `test_2 db/test_2.wal.db` | 192.4 ± 30.4 | 164.0 | 259.9 | 1.96 ± 0.31 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_3` | 150.5 ± 24.8 | 125.5 | 186.1 | 1.00 |
| `test_3 db/test_3.db` | 4062.4 ± 271.3 | 3654.4 | 4634.7 | 27.00 ± 4.81 |
| `test_3 db/test_3.ring.db` | 5786.9 ± 221.8 | 5373.7 | 6080.0 | 38.45 ± 6.51 |
| `test_3 db/test_3.ring.wal.db` | 5255.5 ± 465.5 | 4633.1 | 6161.7 | 34.92 ± 6.54 |
| `test_3 db/test_3.wal.db` | 3534.8 ± 236.0 | 3205.1 | 3941.8 | 23.49 ± 4.18 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_4` | 96.9 ± 0.8 | 95.6 | 99.9 | 1.00 |
| `test_4 db/test_4.db` | 149.5 ± 18.9 | 119.5 | 190.2 | 1.54 ± 0.20 |
| `test_4 db/test_4.ring.db` | 135.1 ± 10.0 | 128.4 | 166.4 | 1.39 ± 0.10 |
| `test_4 db/test_4.ring.wal.db` | 131.9 ± 3.2 | 129.0 | 142.4 | 1.36 ± 0.03 |
| `test_4 db/test_4.wal.db` | 167.4 ± 2.2 | 163.4 | 170.2 | 1.73 ± 0.03 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_5` | 100.2 ± 11.4 | 95.8 | 156.2 | 1.00 |
| `test_5 db/test_5.db` | 121.7 ± 2.3 | 115.3 | 126.7 | 1.22 ± 0.14 |
| `test_5 db/test_5.ring.db` | 132.3 ± 5.3 | 128.6 | 152.6 | 1.32 ± 0.16 |
| `test_5 db/test_5.ring.wal.db` | 131.9 ± 1.7 | 128.7 | 135.8 | 1.32 ± 0.15 |
| `test_5 db/test_5.wal.db` | 165.3 ± 4.4 | 148.8 | 169.6 | 1.65 ± 0.19 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_6` | 95.3 ± 1.9 | 94.0 | 104.3 | 1.00 |
| `test_6 db/test_6.db` | 7144.6 ± 409.3 | 6438.6 | 8050.1 | 74.97 ± 4.55 |
| `test_6 db/test_6.ring.db` | 10845.3 ± 682.9 | 10037.5 | 12086.2 | 113.81 ± 7.52 |
| `test_6 db/test_6.ring.wal.db` | 9824.5 ± 457.6 | 8931.9 | 10521.4 | 103.09 ± 5.22 |
| `test_6 db/test_6.wal.db` | 6078.7 ± 330.6 | 5591.8 | 6572.9 | 63.79 ± 3.69 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_7` | 125.5 ± 4.0 | 122.7 | 142.1 | 1.00 |
| `test_7 db/test_7.db` | 3461.0 ± 231.6 | 3128.8 | 4033.8 | 27.59 ± 2.05 |
| `test_7 db/test_7.ring.db` | 4988.5 ± 393.6 | 4464.7 | 5771.1 | 39.76 ± 3.39 |
| `test_7 db/test_7.ring.wal.db` | 4386.0 ± 287.8 | 3839.2 | 4848.0 | 34.96 ± 2.56 |
| `test_7 db/test_7.wal.db` | 2758.9 ± 205.4 | 2436.4 | 3052.6 | 21.99 ± 1.78 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_8` | 645.1 ± 11.5 | 622.3 | 661.6 | 1.00 |
| `test_8 db/test_8.db` | 24049.7 ± 2134.5 | 20911.2 | 27012.6 | 37.28 ± 3.37 |
| `test_8 db/test_8.ring.db` | 74134.3 ± 6544.0 | 64428.8 | 83722.0 | 114.93 ± 10.35 |
| `test_8 db/test_8.ring.wal.db` | 46114.0 ± 6595.8 | 36568.0 | 55722.2 | 71.49 ± 10.30 |
| `test_8 db/test_8.wal.db` | 14945.2 ± 2118.4 | 12069.9 | 18285.6 | 23.17 ± 3.31 |

| Command | Mean [s] | Min [s] | Max [s] | Relative |
| --- | --- | --- | --- | --- |
| `test_9` | 1.412 ± 0.017 | 1.385 | 1.442 | 1.00 |
| `test_9 db/test_9.db` | 12.207 ± 4.533 | 5.460 | 18.856 | 8.64 ± 3.21 |
| `test_9 db/test_9.ring.db` | 12.158 ± 4.376 | 5.542 | 18.552 | 8.61 ± 3.10 |
| `test_9 db/test_9.ring.wal.db` | 12.206 ± 4.383 | 5.726 | 18.597 | 8.64 ± 3.11 |
| `test_9 db/test_9.wal.db` | 12.195 ± 4.370 | 5.635 | 18.386 | 8.63 ± 3.10 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_10` | 40.7 ± 1.3 | 40.0 | 48.9 | 1.00 |
| `test_10 db/test_10.db` | 810.3 ± 45.6 | 741.3 | 880.1 | 19.93 ± 1.28 |
| `test_10 db/test_10.ring.db` | 1058.8 ± 61.5 | 963.6 | 1158.3 | 26.04 ± 1.72 |
| `test_10 db/test_10.ring.wal.db` | 843.9 ± 62.7 | 751.4 | 928.9 | 20.76 ± 1.67 |
| `test_10 db/test_10.wal.db` | 626.8 ± 42.3 | 566.5 | 684.2 | 15.42 ± 1.15 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_11` | 9.6 ± 0.2 | 9.3 | 11.6 | 1.00 |
| `test_11 db/test_11.db` | 20.8 ± 0.5 | 20.0 | 22.8 | 2.16 ± 0.08 |
| `test_11 db/test_11.ring.db` | 27.8 ± 5.7 | 20.7 | 52.8 | 2.88 ± 0.59 |
| `test_11 db/test_11.ring.wal.db` | 26.4 ± 8.8 | 21.0 | 89.2 | 2.74 ± 0.92 |
| `test_11 db/test_11.wal.db` | 25.9 ± 0.7 | 22.6 | 31.0 | 2.69 ± 0.10 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_12` | 22.4 ± 0.6 | 21.8 | 25.9 | 1.00 |
| `test_12 db/test_12.db` | 70.1 ± 19.7 | 40.9 | 179.4 | 3.13 ± 0.88 |
| `test_12 db/test_12.ring.db` | 107.5 ± 31.5 | 52.1 | 161.9 | 4.80 ± 1.41 |
| `test_12 db/test_12.ring.wal.db` | 111.2 ± 34.0 | 49.8 | 173.8 | 4.97 ± 1.53 |
| `test_12 db/test_12.wal.db` | 69.3 ± 14.2 | 42.4 | 94.5 | 3.10 ± 0.64 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_13` | 26.3 ± 0.5 | 25.9 | 29.0 | 1.00 |
| `test_13 db/test_13.db` | 394.9 ± 235.7 | 49.8 | 788.3 | 15.00 ± 8.96 |
| `test_13 db/test_13.ring.db` | 375.3 ± 235.3 | 68.3 | 845.4 | 14.26 ± 8.94 |
| `test_13 db/test_13.ring.wal.db` | 375.6 ± 234.6 | 70.3 | 810.0 | 14.27 ± 8.92 |
| `test_13 db/test_13.wal.db` | 330.9 ± 195.5 | 62.2 | 675.6 | 12.57 ± 7.43 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_14` | 34.5 ± 0.6 | 33.8 | 36.7 | 1.00 |
| `test_14 db/test_14.db` | 484.5 ± 243.8 | 118.3 | 1015.3 | 14.06 ± 7.08 |
| `test_14 db/test_14.ring.db` | 494.4 ± 170.9 | 247.9 | 776.9 | 14.35 ± 4.97 |
| `test_14 db/test_14.ring.wal.db` | 526.9 ± 223.5 | 202.0 | 895.1 | 15.29 ± 6.49 |
| `test_14 db/test_14.wal.db` | 410.6 ± 130.4 | 232.0 | 609.7 | 11.92 ± 3.79 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_15` | 34.7 ± 0.8 | 34.0 | 38.8 | 1.00 |
| `test_15 db/test_15.db` | 60.8 ± 1.3 | 56.8 | 63.5 | 1.75 ± 0.05 |
| `test_15 db/test_15.ring.db` | 73.9 ± 3.2 | 69.9 | 87.3 | 2.13 ± 0.10 |
| `test_15 db/test_15.ring.wal.db` | 74.4 ± 6.0 | 68.9 | 109.6 | 2.15 ± 0.18 |
| `test_15 db/test_15.wal.db` | 62.5 ± 0.9 | 60.4 | 65.1 | 1.80 ± 0.05 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_16` | 33.4 ± 0.6 | 32.6 | 36.3 | 1.00 |
| `test_16 db/test_16.db` | 47.9 ± 3.8 | 44.7 | 71.9 | 1.43 ± 0.12 |
| `test_16 db/test_16.ring.db` | 53.2 ± 1.3 | 51.2 | 56.3 | 1.59 ± 0.05 |
| `test_16 db/test_16.ring.wal.db` | 53.7 ± 1.9 | 51.0 | 64.1 | 1.61 ± 0.07 |
| `test_16 db/test_16.wal.db` | 64.0 ± 1.2 | 61.6 | 66.5 | 1.91 ± 0.05 |

### x86_64 intel, VirtualBox

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_1` | 10.4 ± 9.4 | 8.6 | 153.4 | 1.00 |
| `test_1 db/test_1.db` | 4630.9 ± 531.5 | 4197.1 | 5561.1 | 447.40 ± 410.79 |
| `test_1 db/test_1.ring.db` | 1951.1 ± 685.1 | 1213.0 | 3503.8 | 188.50 ± 184.03 |
| `test_1 db/test_1.ring.wal.db` | 2769.1 ± 1240.6 | 1346.5 | 4854.9 | 267.53 ± 271.59 |
| `test_1 db/test_1.wal.db` | 2241.6 ± 183.8 | 2015.5 | 2596.8 | 216.56 ± 198.08 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_2` | 371.4 ± 66.4 | 247.1 | 420.9 | 1.03 ± 0.29 |
| `test_2 db/test_2.db` | 435.4 ± 46.6 | 373.8 | 553.5 | 1.21 ± 0.29 |
| `test_2 db/test_2.ring.db` | 397.4 ± 89.9 | 276.8 | 487.1 | 1.11 ± 0.34 |
| `test_2 db/test_2.ring.wal.db` | 472.7 ± 36.7 | 423.8 | 548.1 | 1.31 ± 0.30 |
| `test_2 db/test_2.wal.db` | 359.4 ± 76.1 | 257.9 | 450.0 | 1.00 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_3` | 527.9 ± 57.4 | 461.1 | 658.5 | 1.00 |
| `test_3 db/test_3.db` | 1454.1 ± 180.4 | 1249.1 | 1760.7 | 2.75 ± 0.45 |
| `test_3 db/test_3.ring.db` | 4028.8 ± 1773.4 | 1361.9 | 6070.2 | 7.63 ± 3.46 |
| `test_3 db/test_3.ring.wal.db` | 3849.9 ± 2358.4 | 675.9 | 7601.8 | 7.29 ± 4.54 |
| `test_3 db/test_3.wal.db` | 1032.9 ± 310.6 | 518.2 | 1313.2 | 1.96 ± 0.63 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_4` | 382.1 ± 47.0 | 245.2 | 417.7 | 1.00 |
| `test_4 db/test_4.db` | 385.4 ± 58.8 | 258.0 | 441.8 | 1.01 ± 0.20 |
| `test_4 db/test_4.ring.db` | 430.9 ± 35.5 | 375.4 | 489.9 | 1.13 ± 0.17 |
| `test_4 db/test_4.ring.wal.db` | 427.9 ± 82.9 | 264.6 | 554.9 | 1.12 ± 0.26 |
| `test_4 db/test_4.wal.db` | 420.7 ± 24.6 | 368.7 | 446.2 | 1.10 ± 0.15 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_5` | 326.6 ± 64.7 | 226.6 | 385.4 | 1.00 |
| `test_5 db/test_5.db` | 391.5 ± 23.1 | 339.2 | 419.6 | 1.20 ± 0.25 |
| `test_5 db/test_5.ring.db` | 387.9 ± 89.0 | 274.0 | 486.3 | 1.19 ± 0.36 |
| `test_5 db/test_5.ring.wal.db` | 441.9 ± 27.2 | 406.4 | 484.0 | 1.35 ± 0.28 |
| `test_5 db/test_5.wal.db` | 344.8 ± 78.3 | 251.2 | 448.1 | 1.06 ± 0.32 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_6` | 396.6 ± 43.5 | 345.7 | 506.1 | 1.00 |
| `test_6 db/test_6.db` | 1856.5 ± 542.8 | 1206.1 | 3224.9 | 4.68 ± 1.46 |
| `test_6 db/test_6.ring.db` | 5879.5 ± 3489.1 | 2376.7 | 10758.2 | 14.82 ± 8.95 |
| `test_6 db/test_6.ring.wal.db` | 3898.6 ± 2451.7 | 788.7 | 9459.3 | 9.83 ± 6.28 |
| `test_6 db/test_6.wal.db` | 1084.6 ± 240.4 | 677.9 | 1388.0 | 2.73 ± 0.68 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_7` | 494.5 ± 68.5 | 434.7 | 660.2 | 1.00 |
| `test_7 db/test_7.db` | 1106.8 ± 152.7 | 803.0 | 1282.8 | 2.24 ± 0.44 |
| `test_7 db/test_7.ring.db` | 2731.3 ± 1946.5 | 697.8 | 5350.9 | 5.52 ± 4.01 |
| `test_7 db/test_7.ring.wal.db` | 2601.2 ± 1627.5 | 623.4 | 5079.9 | 5.26 ± 3.37 |
| `test_7 db/test_7.wal.db` | 822.0 ± 209.7 | 564.6 | 1228.1 | 1.66 ± 0.48 |

| Command | Mean [s] | Min [s] | Max [s] | Relative |
| --- | --- | --- | --- | --- |
| `test_8` | 2.814 ± 0.281 | 2.314 | 3.102 | 1.00 |
| `test_8 db/test_8.db` | 24.036 ± 8.930 | 11.041 | 37.724 | 8.54 ± 3.29 |
| `test_8 db/test_8.ring.db` | 23.984 ± 9.013 | 11.103 | 38.410 | 8.52 ± 3.31 |
| `test_8 db/test_8.ring.wal.db` | 24.274 ± 9.171 | 11.133 | 38.712 | 8.63 ± 3.37 |
| `test_8 db/test_8.wal.db` | 24.158 ± 8.908 | 11.472 | 37.448 | 8.59 ± 3.28 |

| Command | Mean [s] | Min [s] | Max [s] | Relative |
| --- | --- | --- | --- | --- |
| `test_9` | 6.269 ± 0.283 | 5.881 | 6.623 | 1.00 |
| `test_9 db/test_9.db` | 52.981 ± 19.343 | 24.574 | 82.915 | 8.45 ± 3.11 |
| `test_9 db/test_9.ring.db` | 53.092 ± 19.455 | 24.432 | 83.084 | 8.47 ± 3.13 |
| `test_9 db/test_9.ring.wal.db` | 53.961 ± 20.785 | 24.343 | 86.876 | 8.61 ± 3.34 |
| `test_9 db/test_9.wal.db` | 55.347 ± 18.596 | 26.589 | 79.522 | 8.83 ± 2.99 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_10` | 168.6 ± 32.5 | 115.4 | 216.5 | 1.00 |
| `test_10 db/test_10.db` | 272.7 ± 28.7 | 204.9 | 320.7 | 1.62 ± 0.36 |
| `test_10 db/test_10.ring.db` | 373.0 ± 125.0 | 188.3 | 555.4 | 2.21 ± 0.86 |
| `test_10 db/test_10.ring.wal.db` | 371.2 ± 74.0 | 264.3 | 514.3 | 2.20 ± 0.61 |
| `test_10 db/test_10.wal.db` | 234.3 ± 49.2 | 159.4 | 292.6 | 1.39 ± 0.40 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_11` | 41.6 ± 7.9 | 26.8 | 98.6 | 1.00 |
| `test_11 db/test_11.db` | 70.8 ± 6.8 | 57.4 | 87.9 | 1.70 ± 0.36 |
| `test_11 db/test_11.ring.db` | 67.7 ± 12.1 | 47.6 | 86.0 | 1.63 ± 0.43 |
| `test_11 db/test_11.ring.wal.db` | 78.5 ± 6.9 | 54.9 | 84.4 | 1.89 ± 0.40 |
| `test_11 db/test_11.wal.db` | 55.3 ± 5.3 | 34.7 | 63.7 | 1.33 ± 0.28 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_12` | 77.1 ± 35.6 | 50.2 | 267.4 | 1.00 |
| `test_12 db/test_12.db` | 161.1 ± 41.7 | 74.9 | 211.9 | 2.09 ± 1.11 |
| `test_12 db/test_12.ring.db` | 148.1 ± 38.6 | 109.8 | 246.6 | 1.92 ± 1.02 |
| `test_12 db/test_12.ring.wal.db` | 179.8 ± 25.1 | 119.6 | 212.5 | 2.33 ± 1.12 |
| `test_12 db/test_12.wal.db` | 137.6 ± 22.2 | 105.1 | 188.5 | 1.78 ± 0.87 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_13` | 101.9 ± 14.4 | 63.5 | 117.8 | 1.00 |
| `test_13 db/test_13.db` | 185.0 ± 30.1 | 128.7 | 245.4 | 1.81 ± 0.39 |
| `test_13 db/test_13.ring.db` | 250.9 ± 83.4 | 160.5 | 381.9 | 2.46 ± 0.89 |
| `test_13 db/test_13.ring.wal.db` | 332.1 ± 106.0 | 186.7 | 504.2 | 3.26 ± 1.14 |
| `test_13 db/test_13.wal.db` | 168.7 ± 19.3 | 137.1 | 199.7 | 1.65 ± 0.30 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_14` | 148.4 ± 14.5 | 108.1 | 166.5 | 1.00 |
| `test_14 db/test_14.db` | 334.2 ± 61.9 | 249.5 | 448.3 | 2.25 ± 0.47 |
| `test_14 db/test_14.ring.db` | 624.7 ± 276.3 | 329.0 | 1044.9 | 4.21 ± 1.91 |
| `test_14 db/test_14.ring.wal.db` | 606.2 ± 177.9 | 339.3 | 926.0 | 4.08 ± 1.26 |
| `test_14 db/test_14.wal.db` | 339.9 ± 86.7 | 214.9 | 453.9 | 2.29 ± 0.63 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_15` | 138.9 ± 24.7 | 93.6 | 180.5 | 1.00 |
| `test_15 db/test_15.db` | 178.4 ± 12.5 | 153.1 | 197.6 | 1.28 ± 0.25 |
| `test_15 db/test_15.ring.db` | 220.3 ± 23.9 | 171.4 | 257.9 | 1.59 ± 0.33 |
| `test_15 db/test_15.ring.wal.db` | 194.6 ± 41.9 | 150.1 | 255.0 | 1.40 ± 0.39 |
| `test_15 db/test_15.wal.db` | 170.9 ± 8.6 | 145.4 | 186.8 | 1.23 ± 0.23 |

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| --- | --- | --- | --- | --- |
| `test_16` | 154.5 ± 17.7 | 127.9 | 217.1 | 1.02 ± 0.25 |
| `test_16 db/test_16.db` | 163.0 ± 42.7 | 119.6 | 306.8 | 1.08 ± 0.37 |
| `test_16 db/test_16.ring.db` | 200.4 ± 20.3 | 145.3 | 225.0 | 1.32 ± 0.32 |
| `test_16 db/test_16.ring.wal.db` | 196.2 ± 17.9 | 144.0 | 218.7 | 1.29 ± 0.30 |
| `test_16 db/test_16.wal.db` | 151.6 ± 32.8 | 116.3 | 198.7 | 1.00 |


## Rollback Journalling
Rollback Journal + Unix VFS, is the fastest on every test.

## WAL
WAL on IO Uring VFS has some competitive edge, less than half, but not on all tests.

## Conclusion

***Warning***: Do not trust these numbers, run them yourself. These tests ran on "noisy" virtualized / containerized machines.

For speed, there is no reason to use IO Uring at all, with this specific implementation.
Maybe except for those specific cases were IO Uring + WAL seems to have a competitive edge.

## Future research ideas
* Release build, speed difference?
* Implement on [windows IoRing](https://learn.microsoft.com/en-us/windows/win32/api/ioringapi/)
* Apply insert optimizations [mentioned here](https://voidstar.tech/sqlite_insert_speed) 
* Vfs consensus via IO Uring (IO Uring) sockets + Raft, e.g. rqlite
* Turn on libc::O_DIRECT as u64 | libc::O_SYNC as u64 on storage devices that support it
* Reimplement everything with Zig + liburing

## Determine whether your kernel supports IO Uring

Linux command-line:
1. uname -a # expect 5 and above
2. grep io_uring_setup /proc/kallsyms # expect 2 lines
3. gcc test_io_uring.c -o test_io_uring && ./test_io_uring

```C
// test_io_uring.c

#include <stdio.h>
#include <errno.h>
#include <linux/io_uring.h>
#include <stddef.h>
#include <sys/syscall.h>
#include <unistd.h>

int main(int argc, char **argv) {
  if (syscall(__NR_io_uring_register, 0, IORING_UNREGISTER_BUFFERS, NULL, 0) && errno == ENOSYS) {
    printf("%s", "nope\n");
    return -1;
  } else {
    printf("%s", "yep\n");
    return 0;
  }
}

```

