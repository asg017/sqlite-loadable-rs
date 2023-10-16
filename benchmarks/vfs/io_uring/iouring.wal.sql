.mode box
.header on

.load target/debug/lib_iouringvfs

SELECT io_uring_vfs_from_file('iouring.ext.wal.db');

ATTACH io_uring_vfs_from_file('iouring.ext.wal.db') AS "iouring.ext.wal";

-- PRAGMA locking_mode = NORMAL;

CREATE TABLE t3(x varchar(10), y integer);

INSERT INTO t3 VALUES('a', 4),
                     ('b', 5),
                     ('c', 3),
                     ('d', 8),
                     ('e', 1);
