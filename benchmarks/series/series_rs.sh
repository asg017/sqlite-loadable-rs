#!/bin/bash

sqlite3x :memory: '.load ../target/series_rs' "select count(value) from generate_series_rs(1, $1);"