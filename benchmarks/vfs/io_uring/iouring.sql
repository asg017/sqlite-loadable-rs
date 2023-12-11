-- HOWTO
-- 0. Make sure the sqlite3 was built with the build tag: SQLITE3VFS_LOADABLE_EXT
-- 1. In Cargo.toml, disable the 'static' feature, from the sqlite-loadable library
--   it should look like this: sqlite-loadable = {path="../../../"}
-- 2. Cargo build
-- 3. load this script: sqlite3 --init iouring.sql

-- This script was tested on 3.44.2, compiled with:
--   gcc -g -DSQLITE_DEBUG shell.c sqlite3.c -lpthread -ldl -o sqlite3

.mode box
.header on

.load target/debug/lib_iouringvfs

--ATTACH io_uring_vfs_from_file('iouring.db') AS "iouring0";
--SELECT io_uring_vfs_from_file('iouring.db');

ATTACH 'file:iouring.db?vfs=iouring' as iouring;

.open "iouring.db"

.vfslist

CREATE TABLE IF NOT EXISTS t3(x varchar(10), y integer);

INSERT INTO t3 VALUES('a', 4),
                     ('b', 5),
                     ('c', 3),
                     ('d', 8),
                     ('e', 1);
