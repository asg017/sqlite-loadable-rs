#include "sqlite3ext.h"

SQLITE_EXTENSION_INIT1

static void yo(sqlite3_context *context, int argc, sqlite3_value **argv) {
  sqlite3_result_text(context, "yo", -1, SQLITE_STATIC);;
}

static void surround(sqlite3_context *context, int argc, sqlite3_value **argv) {
  char * s = sqlite3_mprintf("x%sx", sqlite3_value_text(argv[0]));
  sqlite3_result_text(context, s, -1, SQLITE_TRANSIENT);
  sqlite3_free(s);
}

static void add(sqlite3_context *context, int argc, sqlite3_value **argv) {
  int a = sqlite3_value_int(argv[0]);
  int b = sqlite3_value_int(argv[1]);
  sqlite3_result_int(context, a + b);
}


#ifdef _WIN32
__declspec(dllexport)
#endif
int sqlite3_scalarc_init(sqlite3 *db, char **pzErrMsg, const sqlite3_api_routines *pApi) {
  SQLITE_EXTENSION_INIT2(pApi);
  sqlite3_create_function(db, "surround_c", 1, SQLITE_DETERMINISTIC, 0, surround, 0, 0);
  sqlite3_create_function(db, "yo_c", 0, SQLITE_DETERMINISTIC, 0, yo, 0, 0);
  sqlite3_create_function(db, "add_c", 2, SQLITE_DETERMINISTIC, 0, yo, 0, 0);
  return 0;
}