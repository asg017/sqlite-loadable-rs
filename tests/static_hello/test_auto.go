package main

import (
	"database/sql"
	"fmt"
	"log"

	_ "github.com/mattn/go-sqlite3"
)

// #cgo darwin,amd64 LDFLAGS: -Wl,-undefined,dynamic_lookup -lhello
// #cgo darwin,arm64 LDFLAGS: -Wl,-undefined,dynamic_lookup -lhello
// #cgo CFLAGS: -DSQLITE_CORE
// #include <sqlite3ext.h>
// #include "hello.h"
import "C"

func main() {
	C.sqlite3_auto_extension( (*[0]byte) ((C.sqlite3_hello_init)) );

	db, err := sql.Open("sqlite3", ":memory:")
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	var result string
	err = db.QueryRow("SELECT hello(?)", "asdf").Scan(&result)
	if err != nil {
		log.Fatal(err)
	}

	if result != "hello, asdf!" {
		panic(result)
	}

	fmt.Println("✅ demo.go ran successfully. \n");
}
