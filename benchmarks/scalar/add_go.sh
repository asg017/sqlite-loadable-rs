#!/bin/bash
sqlite3x :memory: '.load ../target/scalar_go' "select add_go(value, value) from generate_series(1, $1);"
