#!/bin/sh
NAME="sqlite-loadable-rs:1.0"
docker image inspect "$NAME" || docker build -t "$NAME" .
docker run -it -p 2222:22 -v $PWD:/root -w /root $NAME

# see https://github.com/jfrimmel/cargo-valgrind/pull/58/commits/1c168f296e0b3daa50279c642dd37aecbd85c5ff#L59
# scan for double frees and leaks
# VALGRINDFLAGS="--leak-check=yes --trace-children=yes" cargo valgrind test
