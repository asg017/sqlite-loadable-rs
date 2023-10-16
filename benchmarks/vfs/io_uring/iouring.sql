.mode box
.header on

.load target/debug/lib_iouringvfs
.eqp full
-- trace not supported in version: 3.40.1 2022-12-28 14:03:47 df5c253c0b3dd24916e4ec7cf77d3db5294cc9fd45ae7b9c5e82ad8197f3alt1

SELECT io_uring_vfs_from_file('iouring-ext.db');

ATTACH io_uring_vfs_from_file('iouring-ext.db') AS "iouring-ext";

.open "iouring-ext.db"

.vfslist

CREATE TABLE t3(x varchar(10), y integer);

INSERT INTO t3 VALUES('a', 4),
                     ('b', 5),
                     ('c', 3),
                     ('d', 8),
                     ('e', 1);
