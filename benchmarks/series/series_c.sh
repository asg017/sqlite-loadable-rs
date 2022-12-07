#!/bin/bash

sqlite3x :memory: '.load ../target/series_c' "select count(value) from generate_series_c(1, $1);"