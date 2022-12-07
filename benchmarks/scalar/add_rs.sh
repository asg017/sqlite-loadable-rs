#!/bin/bash

sqlite3x :memory: '.load ../target/scalar_rs' "select add_rs(value, value) from generate_series(1, $1);"