# sqlite3_vfs_io_uring_rs
PoC: sqlite3 vfs extension support for IO Uring

Warning: IO Uring is only supported on linux, where this is turned on.
IO Uring has been turned off on many distros due to certain security issues.

This project was tested on Docker and VirtualBox.

## Benchmark speeds with hyperfine

### How
Run this script in a shell
```bash
sh run-hyperfine.sh
```

### Conclusion
Running on memory runs the fastest in all tests.
As expected running on IO Uring holds the 2nd place,
file ops via system calls comes in last, in most tests.

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
