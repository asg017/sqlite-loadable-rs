.mode box
.header on

.load target/debug/examples/libmem_vfs

select memvfs_from_file('from.db');

#ATTACH memvfs_from_file('from.db') AS inmem;
#memvfs_to_file("to.db")
