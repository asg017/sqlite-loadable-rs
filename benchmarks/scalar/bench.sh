#!/bin/bash

end=1e6
bench_scalar_yo() {
  hyperfine --warmup 10 --export-json=results-yo.json \
    "./yo_c.sh $end" \
    "./yo_rs.sh $end" \
    "./yo_go.sh $end" 
}

bench_scalar_surround() {
  hyperfine --warmup 10 --export-json=results-surround.json \
    "./surround_c.sh" \
    "./surround_rs.sh" \
    "./surround_go.sh" 
}

bench_scalar_add() {
  hyperfine --warmup 10 --export-json=results-add.json \
    "./add_c.sh $end" \
    "./add_rs.sh $end" \
    "./add_go.sh $end" 
}

main() {
  bench_scalar_yo;
  bench_scalar_surround;
  bench_scalar_add;
}

main
