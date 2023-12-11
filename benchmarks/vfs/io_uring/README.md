# sqlite3_vfs_io_uring_rs
Performance test: sqlite3 vfs + IO Uring with WAL and rollback journalling

Warning: IO Uring is only supported on linux, where this IO Uring has been activated.
IO Uring has been turned off on many distros due to certain security issues.

This project was tested on Docker and VirtualBox. Your mileage will vary.
Also, all of the tests ran on [rusqlite](https://github.com/rusqlite/rusqlite).

## Benchmark speeds with hyperfine

[This script](./run-hyperfine.sh) was written to benchmark and compare, memory vfs as baseline, unix vfs and
the custom IO Uring based vfs, with the default [rollback journalling, and WAL](https://fly.io/blog/sqlite-internals-wal/).

### Tests

Tests were [derived from this archived sqlite document](https://www.sqlite.org/speed.html),
to show whether adding IO Uring support to a custom IO Uring vfs will impact sqlite3's performance.

16 tests are run, on volatile memory, file storage and file storage via io-uring, where memory storage serves as a baseline/control.

### Run the tests
Run [this script](./run-hyperfine.sh) in a shell
```bash
sh run-hyperfine.sh
```

If you don't have linux running on your machine (yet), use
[the docker script provided here](../../../run-docker.sh).

### Logging

```bash
RUST_LOG=trace cargo test
```

### Results

The numbers here were generated on a noisy machine.
Your mileage might vary.

| Test | Desc | 2nd Winner |
| --- | --- |
| 1 | INSERTs |
| 2 | INSERTs in a transaction |
| 3 | INSERTs into an indexed table |
| 4 | SELECTs without an index |
| 5 | SELECTs on a string comparison |
| 6 | Creating an index |
| 7 | SELECTs with an index |
| 8 | UPDATEs without an index |
| 9 | UPDATEs with an index |
| 10 | Text UPDATEs with an index |
| 11 | INSERTs from a SELECT |
| 12 | DELETE without an index |
| 13 | DELETE with an index |
| 14 | A big INSERT after a big DELETE |
| 15 | A big DELETE followed by many small INSERTs |
| 16 | DROP TABLE |

## Conclusion

TODO

## Future research ideas
* Release build, speed difference?
* Implement on [windows IoRing](https://learn.microsoft.com/en-us/windows/win32/api/ioringapi/)
* Apply insert optimizations [mentioned here](https://voidstar.tech/sqlite_insert_speed) 
* Vfs consensus via IO Uring (IO Uring) sockets + Raft, e.g. rqlite
* Turn on libc::O_DIRECT as u64 | libc::O_SYNC as u64 on storage devices that support it

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

## Results UNIX VFS vs IO Uring VFS, with memory VFS as baseline

