#!/bin/bash
sqlite3x :memory: '.load ../target/scalar_c' "select add_c(value, value) from generate_series(1, $1);"
