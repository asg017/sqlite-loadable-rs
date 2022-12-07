#!/bin/bash

sqlite3x :memory: '.load ../target/scalar_rs' "select count(yo_rs()) from generate_series(1, $1);"