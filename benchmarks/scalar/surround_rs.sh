#!/bin/bash
sqlite3x words.db '.load ../target/scalar_rs' "select count(surround_rs(word)) from words;"