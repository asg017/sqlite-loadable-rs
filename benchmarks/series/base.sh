#!/bin/bash

sqlite3x :memory: "select count(value) from generate_series(1, $1);"