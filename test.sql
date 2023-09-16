.mode box
.header on

.load target/debug/examples/libmem_vfs

select memvfs_from_file('from.db');

ATTACH memvfs_from_file('from.db') AS inmem;

CREATE TABLE t3(x, y);

INSERT INTO t3 VALUES('a', 4),
                     ('b', 5),
                     ('c', 3),
                     ('d', 8),
                     ('e', 1);
