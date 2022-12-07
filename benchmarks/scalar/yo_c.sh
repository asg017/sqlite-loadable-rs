#!/bin/bash
sqlite3x :memory: '.load ../target/scalar_c' "select count(yo_c()) from generate_series(1, $1);"