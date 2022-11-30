-- sqlite3 :memory: '.read examples/test.sql'

.bail on

.header on
--.mode box

.load target/release/examples/libhello

select
  hello('world'),
  hello('Alex'),
  hello(1234);

select * from pragma_function_list where name = 'hello';


.load target/release/examples/libseries sqlite3_seriesrs_init

.timer on
select count(value) as rs from generate_series_rs(1, 1e7);

select count(value) as c from generate_series(1, 1e7);