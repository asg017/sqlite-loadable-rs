#!/bin/bash
sqlite3x words.db '.load ../target/scalar_c' "select count(surround_c(word)) from words;"
