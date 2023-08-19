.mode box
.header on

.load target/debug/examples/libin

select *
from vtab_in('xxx')
where y in (select value from json_each('[1,2,"alex", "puppy"]'));
