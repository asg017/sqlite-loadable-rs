.mode box
.header on

.load target/debug/examples/libmem_vfs

SELECT mem_vfs_uri();

ATTACH mem_vfs_uri() AS inmem;

-- attach does not actually do anything
.open ___mem___ -- does

CREATE TABLE t3(x, y);

INSERT INTO t3 VALUES('a', 4),
                     ('b', 5),
                     ('c', 3),
                     ('d', 8),
                     ('e', 1);
