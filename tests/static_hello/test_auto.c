#include "sqlite3.h"
#include "hello.h"
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <assert.h>

int main(int argc, char *argv[]) {
  int rc = SQLITE_OK;
  sqlite3 *db;
  sqlite3_stmt *stmt;

  rc = sqlite3_auto_extension((void (*)())sqlite3_hello_init);
  assert(rc == SQLITE_OK);

  rc = sqlite3_open(":memory:", &db);
  assert(rc == SQLITE_OK);

  rc = sqlite3_prepare_v2(db, "SELECT hello('asdf')", -1, &stmt, NULL);
  assert(rc == SQLITE_OK);
  rc = sqlite3_step(stmt);
  assert(rc == SQLITE_ROW);

  assert(strcmp((const char *) sqlite3_column_text(stmt, 0), "hello, asdf!") == 0);

  printf("✅ demo.c ran successfully. \n");

  sqlite3_finalize(stmt);
  sqlite3_close(db);
  return 0;
}
