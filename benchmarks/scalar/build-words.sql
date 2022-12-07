.load ../../../sqlite-lines/dist/lines0
create table words as select line as word from lines_read('/usr/share/dict/words');
