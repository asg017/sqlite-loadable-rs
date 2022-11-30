#!/bin/bash

end=1e3
bench_scalar_yo() {
  hyperfine --warmup 8 --export-json=benchmarks/scalar/results-yo.json \
    "sqlite3x :memory: '.load benchmarks/target/scalar_c' 'select yo_c() from generate_series(1, $end);'" \
    "sqlite3x :memory: '.load benchmarks/target/scalar_rs' 'select yo_rs() from generate_series(1, $end);'" 
}

bench_scalar_surround() {
  hyperfine --warmup 8 --export-json=benchmarks/scalar/results-surround.json \
    "sqlite3x :memory: '.load benchmarks/target/scalar_c' 'select surround_c(\"a\") from generate_series(1, $end);'" \
    "sqlite3x :memory: '.load benchmarks/target/scalar_rs' 'select surround_rs(\"a\") from generate_series(1, $end);'" 
}

bench_scalar_add() {
  hyperfine --warmup 8 --export-json=benchmarks/scalar/results-add.json \
    "sqlite3x :memory: '.load benchmarks/target/scalar_c' 'select add_c(value, value) from generate_series(1, $end);'" \
    "sqlite3x :memory: '.load benchmarks/target/scalar_rs' 'select add_rs(value, value) from generate_series(1, $end);'" \
    "sqlite3x :memory: '.load benchmarks/target/scalar_go' 'select add_go(value, value) from generate_series(1, $end);'" 
}

main() {
  bench_scalar_yo;
  bench_scalar_surround;
  bench_scalar_add;
}

main
  #\
  #"sqlite3x :memory: '.load benchmarks/target/series_go' 'select count(value) from generate_series_go(1, $end);'"