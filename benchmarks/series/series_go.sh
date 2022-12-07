#!/bin/bash

sqlite3x :memory: '.load ../target/series_go' "select count(value) from generate_series_go(1, $1);"