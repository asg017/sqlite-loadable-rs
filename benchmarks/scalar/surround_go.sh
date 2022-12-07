#!/bin/bash
sqlite3x words.db '.load ../target/scalar_go' "select count(surround_go(word)) from words;"
