#!/bin/bash

end=1e6
hyperfine --warmup 8 --export-json=benchmarks/series/results.json \
  "sqlite3x :memory: 'select count(value) from generate_series(1, $end);'" \
  "sqlite3x :memory: '.load benchmarks/target/series_c' 'select count(value) from generate_series_c(1, $end);'" \
  "sqlite3x :memory: '.load benchmarks/target/series_rs' 'select count(value) from generate_series_rs(1, $end);'" #\
  #"sqlite3x :memory: '.load benchmarks/target/series_go' 'select count(value) from generate_series_go(1, $end);'"