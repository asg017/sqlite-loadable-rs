#!/bin/bash
sqlite3x :memory: '.load ../target/scalar_go' "select count(yo_go()) from generate_series(1, $1)"