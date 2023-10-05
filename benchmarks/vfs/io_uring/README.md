# sqlite3_vfs_io_uring_rs
PoC: sqlite3 vfs extension support for IO Uring

Warning: IO Uring is only supported on linux, where this is turned on.
IO Uring has been turned off on many distros due to certain security issues.

This project was tested on Docker and VirtualBox. Your mileage will vary.

## Benchmark speeds with hyperfine

### What
Tests were [derived from this archived sqlite document](https://www.sqlite.org/speed.html).

16 tests are run, on in-mem, in-file and io-uring, where in-memory serves as a baseline/control.

Practially, what io-uring does is circumvent multiple os system-calls to CRUD a file,
whereas the traditional in-file way does it via system-calls.

### How
Run [this script](./run-hyperfine.sh) in a shell
```bash
sh run-hyperfine.sh
```

If you don't have linux running on your machine (yet), use
[the docker script provided here](./run-docker.sh).

### Results

| Test | Desc | Winner |
| --- | --- | --- |
| 1 | 1000 INSERTs | in-file |
| 2 | 25000 INSERTs in a transaction | in-file |
| 3 | 25000 INSERTs into an indexed table | in-file |
| 4 | 100 SELECTs without an index | - |
| 5 | 100 SELECTs on a string comparison | - |
| 6 | Creating an index | - |
| 7 | 5000 SELECTs with an index | - |
| 8 | 1000 UPDATEs without an index | io-uring |
| 9 | 25000 UPDATEs with an index | io-uring |
| 10 | 25000 text UPDATEs with an index | in-file |
| 11 | INSERTs from a SELECT | in-file |
| 12 | DELETE without an index | in-file |
| 13 | DELETE with an index | in-file |
| 14 | A big INSERT after a big DELETE | io-uring |
| 15 | A big DELETE followed by many small INSERTs | in-file |
| 16 | DROP TABLE | io-uring |

The number of executions were reduced due to life being too short.

## Conclusion

It seems that with in-file coming in second on most of the tests,
adding io_uring, to [rusqlite](https://github.com/rusqlite/rusqlite) does not lead to major speed improvements.

## TODO
- [] Fix tests 4 through 7
- [] Use the vfs extension on a production sqlite3 binary

## Other research ideas
* IO Uring storage via paging on Vfs, or multiple file vs single file storage
* All insert optimization mentioned here: https://voidstar.tech/sqlite_insert_speed
* IO Uring and sqlite3 replication via sockets
* Vfs consensus via IO Uring managed sockets + Raft, e.g. rqlite
* Turn on libc::O_DIRECT as u64 | libc::O_SYNC as u64 on drives that support it

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

## Raw benchmark results (on Docker)

Benchmark 1: ./target/debug/examples/test_1
  Time (mean ± σ):       4.5 ms ±   0.1 ms    [User: 2.8 ms, System: 1.0 ms]
  Range (min … max):     4.2 ms …   5.1 ms    580 runs

  Warning: Command took less than 5 ms to complete. Note that the results might be inaccurate because hyperfine can not calibrate the shell startup time much more precise than this limit. You can try to use the `-N`/`--shell=none` option to disable the shell completely.
  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.

Benchmark 2: ./target/debug/examples/test_1 test_1.db
  Time (mean ± σ):      1.350 s ±  0.036 s    [User: 0.036 s, System: 0.291 s]
  Range (min … max):    1.293 s …  1.417 s    10 runs

Benchmark 3: ./target/debug/examples/test_1 test_1.ring.db
  Time (mean ± σ):      1.391 s ±  0.050 s    [User: 0.040 s, System: 0.297 s]
  Range (min … max):    1.317 s …  1.450 s    10 runs

Summary
  ./target/debug/examples/test_1 ran
  303.22 ± 10.93 times faster than ./target/debug/examples/test_1 test_1.db
  312.34 ± 13.46 times faster than ./target/debug/examples/test_1 test_1.ring.db
Benchmark 1: ./target/debug/examples/test_2
  Time (mean ± σ):      97.9 ms ±   1.2 ms    [User: 95.5 ms, System: 1.6 ms]
  Range (min … max):    96.4 ms … 101.7 ms    30 runs

Benchmark 2: ./target/debug/examples/test_2 test_2.db
  Time (mean ± σ):     117.7 ms ±   2.4 ms    [User: 98.9 ms, System: 5.0 ms]
  Range (min … max):   114.0 ms … 122.7 ms    25 runs

Benchmark 3: ./target/debug/examples/test_2 test_2.ring.db
  Time (mean ± σ):     117.9 ms ±   1.9 ms    [User: 99.0 ms, System: 4.7 ms]
  Range (min … max):   114.5 ms … 123.0 ms    26 runs

Summary
  ./target/debug/examples/test_2 ran
    1.20 ± 0.03 times faster than ./target/debug/examples/test_2 test_2.db
    1.20 ± 0.02 times faster than ./target/debug/examples/test_2 test_2.ring.db
Benchmark 1: ./target/debug/examples/test_3
  Time (mean ± σ):     124.1 ms ±   1.1 ms    [User: 121.7 ms, System: 1.7 ms]
  Range (min … max):   122.5 ms … 126.5 ms    23 runs

Benchmark 2: ./target/debug/examples/test_3 test_3.db
  Time (mean ± σ):      1.625 s ±  0.931 s    [User: 0.228 s, System: 0.271 s]
  Range (min … max):    0.217 s …  2.762 s    13 runs

Benchmark 3: ./target/debug/examples/test_3 test_3.ring.db
  Time (mean ± σ):      1.708 s ±  0.940 s    [User: 0.239 s, System: 0.281 s]
  Range (min … max):    0.212 s …  2.825 s    14 runs

Summary
  ./target/debug/examples/test_3 ran
   13.10 ± 7.50 times faster than ./target/debug/examples/test_3 test_3.db
   13.77 ± 7.58 times faster than ./target/debug/examples/test_3 test_3.ring.db

Benchmark 1: ./target/debug/examples/test_4
Error: ExecuteReturnedResults
Error: Command terminated with non-zero exit code: 1. Use the '-i'/'--ignore-failure' option if you want to ignore this. Alternatively, use the '--show-output' option to debug what went wrong.
Benchmark 1: ./target/debug/examples/test_5
Error: ExecuteReturnedResults
Error: Command terminated with non-zero exit code: 1. Use the '-i'/'--ignore-failure' option if you want to ignore this. Alternatively, use the '--show-output' option to debug what went wrong.
Benchmark 1: ./target/debug/examples/test_6
  Time (mean ± σ):      2.051 s ±  0.064 s    [User: 2.017 s, System: 0.032 s]
  Range (min … max):    2.024 s …  2.232 s    10 runs

  Warning: The first benchmarking run for this command was significantly slower than the rest (2.232 s). This could be caused by (filesystem) caches that were not filled until after the first run. You are already using the '--warmup' option which helps to fill these caches before the actual benchmark. You can either try to increase the warmup count further or re-run this benchmark on a quiet system in case it was a random outlier. Alternatively, consider using the '--prepare' option to clear the caches before each timing run.

Benchmark 2: ./target/debug/examples/test_6 test_6.db
Error: SqliteFailure(Error { code: Unknown, extended_code: 1 }, Some("index i6a already exists"))
Error: Command terminated with non-zero exit code: 1. Use the '-i'/'--ignore-failure' option if you want to ignore this. Alternatively, use the '--show-output' option to debug what went wrong.
Benchmark 1: ./target/debug/examples/test_7
Error: ExecuteReturnedResults
Error: Command terminated with non-zero exit code: 1. Use the '-i'/'--ignore-failure' option if you want to ignore this. Alternatively, use the '--show-output' option to debug what went wrong.
Benchmark 1: ./target/debug/examples/test_8
  Time (mean ± σ):     658.0 ms ±   9.7 ms    [User: 655.9 ms, System: 1.2 ms]
  Range (min … max):   640.1 ms … 668.8 ms    10 runs

Benchmark 2: ./target/debug/examples/test_8 test_8.db
  Time (mean ± σ):      4.276 s ±  2.014 s    [User: 4.265 s, System: 0.004 s]
  Range (min … max):    1.334 s …  7.348 s    10 runs

Benchmark 3: ./target/debug/examples/test_8 test_8.ring.db
  Time (mean ± σ):      4.226 s ±  1.972 s    [User: 4.214 s, System: 0.004 s]
  Range (min … max):    1.320 s …  7.283 s    10 runs

Summary
  ./target/debug/examples/test_8 ran
    6.42 ± 3.00 times faster than ./target/debug/examples/test_8 test_8.ring.db
    6.50 ± 3.06 times faster than ./target/debug/examples/test_8 test_8.db
Benchmark 1: ./target/debug/examples/test_9
  Time (mean ± σ):      1.391 s ±  0.020 s    [User: 1.388 s, System: 0.002 s]
  Range (min … max):    1.361 s …  1.425 s    10 runs

Benchmark 2: ./target/debug/examples/test_9 test_9.db
  Time (mean ± σ):      9.217 s ±  4.387 s    [User: 9.207 s, System: 0.004 s]
  Range (min … max):    2.814 s … 15.950 s    10 runs

Benchmark 3: ./target/debug/examples/test_9 test_9.ring.db
  Time (mean ± σ):      9.186 s ±  4.334 s    [User: 9.176 s, System: 0.004 s]
  Range (min … max):    2.781 s … 15.890 s    10 runs

Summary
  ./target/debug/examples/test_9 ran
    6.60 ± 3.12 times faster than ./target/debug/examples/test_9 test_9.ring.db
    6.62 ± 3.15 times faster than ./target/debug/examples/test_9 test_9.db
Benchmark 1: ./target/debug/examples/test_10
  Time (mean ± σ):      40.0 ms ±   0.3 ms    [User: 38.4 ms, System: 1.1 ms]
  Range (min … max):    39.6 ms …  41.2 ms    74 runs

Benchmark 2: ./target/debug/examples/test_10 test_10.db
  Time (mean ± σ):     297.4 ms ± 178.8 ms    [User: 56.3 ms, System: 47.0 ms]
  Range (min … max):    56.8 ms … 650.2 ms    52 runs

Benchmark 3: ./target/debug/examples/test_10 test_10.ring.db
  Time (mean ± σ):     314.6 ms ± 184.4 ms    [User: 58.2 ms, System: 50.0 ms]
  Range (min … max):    56.6 ms … 647.7 ms    52 runs

Summary
  ./target/debug/examples/test_10 ran
    7.43 ± 4.47 times faster than ./target/debug/examples/test_10 test_10.db
    7.86 ± 4.61 times faster than ./target/debug/examples/test_10 test_10.ring.db
Benchmark 1: ./target/debug/examples/test_11
  Time (mean ± σ):       9.2 ms ±   0.1 ms    [User: 7.6 ms, System: 1.0 ms]
  Range (min … max):     8.9 ms …  10.0 ms    312 runs

Benchmark 2: ./target/debug/examples/test_11 test_11.db
  Time (mean ± σ):      18.6 ms ±   1.5 ms    [User: 8.5 ms, System: 2.7 ms]
  Range (min … max):    15.3 ms …  22.4 ms    159 runs

  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.

Benchmark 3: ./target/debug/examples/test_11 test_11.ring.db
  Time (mean ± σ):      20.0 ms ±   2.4 ms    [User: 8.5 ms, System: 3.2 ms]
  Range (min … max):    16.0 ms …  44.8 ms    133 runs

  Warning: The first benchmarking run for this command was significantly slower than the rest (22.2 ms). This could be caused by (filesystem) caches that were not filled until after the first run. You are already using the '--warmup' option which helps to fill these caches before the actual benchmark. You can either try to increase the warmup count further or re-run this benchmark on a quiet system in case it was a random outlier. Alternatively, consider using the '--prepare' option to clear the caches before each timing run.

Summary
  ./target/debug/examples/test_11 ran
    2.02 ± 0.16 times faster than ./target/debug/examples/test_11 test_11.db
    2.17 ± 0.26 times faster than ./target/debug/examples/test_11 test_11.ring.db
Benchmark 1: ./target/debug/examples/test_12
  Time (mean ± σ):      22.2 ms ±   0.9 ms    [User: 20.5 ms, System: 1.1 ms]
  Range (min … max):    21.8 ms …  29.9 ms    133 runs

  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.

Benchmark 2: ./target/debug/examples/test_12 test_12.db
  Time (mean ± σ):      65.5 ms ±  14.7 ms    [User: 42.8 ms, System: 7.5 ms]
  Range (min … max):    35.2 ms …  90.1 ms    75 runs

Benchmark 3: ./target/debug/examples/test_12 test_12.ring.db
  Time (mean ± σ):      69.1 ms ±  16.8 ms    [User: 45.9 ms, System: 7.3 ms]
  Range (min … max):    35.1 ms …  96.4 ms    84 runs

Summary
  ./target/debug/examples/test_12 ran
    2.95 ± 0.67 times faster than ./target/debug/examples/test_12 test_12.db
    3.11 ± 0.77 times faster than ./target/debug/examples/test_12 test_12.ring.db
Benchmark 1: ./target/debug/examples/test_13
  Time (mean ± σ):      25.8 ms ±   0.2 ms    [User: 24.1 ms, System: 1.1 ms]
  Range (min … max):    25.5 ms …  26.5 ms    113 runs

Benchmark 2: ./target/debug/examples/test_13 test_13.db
  Time (mean ± σ):     408.7 ms ± 244.5 ms    [User: 46.4 ms, System: 71.4 ms]
  Range (min … max):    38.8 ms … 842.4 ms    76 runs

Benchmark 3: ./target/debug/examples/test_13 test_13.ring.db
  Time (mean ± σ):     428.4 ms ± 259.2 ms    [User: 47.0 ms, System: 76.5 ms]
  Range (min … max):    39.3 ms … 830.2 ms    75 runs

Summary
  ./target/debug/examples/test_13 ran
   15.83 ± 9.47 times faster than ./target/debug/examples/test_13 test_13.db
   16.60 ± 10.04 times faster than ./target/debug/examples/test_13 test_13.ring.db
Benchmark 1: ./target/debug/examples/test_14
  Time (mean ± σ):      34.1 ms ±   0.3 ms    [User: 32.1 ms, System: 1.3 ms]
  Range (min … max):    33.7 ms …  35.4 ms    87 runs

Benchmark 2: ./target/debug/examples/test_14 test_14.db
  Time (mean ± σ):     543.1 ms ± 301.7 ms    [User: 130.4 ms, System: 92.1 ms]
  Range (min … max):    81.9 ms … 1075.7 ms    36 runs

Benchmark 3: ./target/debug/examples/test_14 test_14.ring.db
  Time (mean ± σ):     536.7 ms ± 291.2 ms    [User: 130.8 ms, System: 90.6 ms]
  Range (min … max):    81.1 ms … 1056.0 ms    36 runs

Summary
  ./target/debug/examples/test_14 ran
   15.73 ± 8.53 times faster than ./target/debug/examples/test_14 test_14.ring.db
   15.91 ± 8.84 times faster than ./target/debug/examples/test_14 test_14.db

Benchmark 1: ./target/debug/examples/test_15
  Time (mean ± σ):      34.1 ms ±   0.3 ms    [User: 32.3 ms, System: 1.1 ms]
  Range (min … max):    33.6 ms …  35.2 ms    86 runs

Benchmark 2: ./target/debug/examples/test_15 test_15.db
  Time (mean ± σ):      58.9 ms ±   3.1 ms    [User: 35.1 ms, System: 5.3 ms]
  Range (min … max):    56.7 ms …  79.9 ms    51 runs

  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.

Benchmark 3: ./target/debug/examples/test_15 test_15.ring.db
  Time (mean ± σ):      59.1 ms ±   0.7 ms    [User: 35.5 ms, System: 5.1 ms]
  Range (min … max):    57.1 ms …  61.1 ms    48 runs

Summary
  ./target/debug/examples/test_15 ran
    1.73 ± 0.09 times faster than ./target/debug/examples/test_15 test_15.db
    1.73 ± 0.03 times faster than ./target/debug/examples/test_15 test_15.ring.db
Benchmark 1: ./target/debug/examples/test_16
  Time (mean ± σ):      32.9 ms ±   0.3 ms    [User: 31.2 ms, System: 1.1 ms]
  Range (min … max):    32.5 ms …  34.5 ms    89 runs

Benchmark 2: ./target/debug/examples/test_16 test_16.db
  Time (mean ± σ):      44.7 ms ±   3.1 ms    [User: 33.5 ms, System: 3.0 ms]
  Range (min … max):    40.1 ms …  53.8 ms    74 runs

Benchmark 3: ./target/debug/examples/test_16 test_16.ring.db
  Time (mean ± σ):      44.5 ms ±   2.0 ms    [User: 33.0 ms, System: 3.1 ms]
  Range (min … max):    39.6 ms …  47.2 ms    74 runs

Summary
  ./target/debug/examples/test_16 ran
    1.35 ± 0.06 times faster than ./target/debug/examples/test_16 test_16.ring.db
    1.36 ± 0.10 times faster than ./target/debug/examples/test_16 test_16.db