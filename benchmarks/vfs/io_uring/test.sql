.mode box
.header on

.load target/debug/lib_iouringvfs

SELECT io_uring_vfs_from_file('from.db');

ATTACH io_uring_vfs_from_file('from.db') AS inring;

CREATE TABLE t3(x, y);

INSERT INTO t3 VALUES('a', 4),
                     ('b', 5),
                     ('c', 3),
                     ('d', 8),
                     ('e', 1);
