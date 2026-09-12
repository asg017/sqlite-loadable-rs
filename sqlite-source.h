/*
** sqlite-source.h — the "source API" contract shared by SQLite extensions.
**
** A *producer* extension (sqlite-fetch, sqlite-objectstore, ...) registers a
** scalar SQL function such as `_http_api()` or `_s3_api()` that returns
**
**     sqlite3_result_pointer(ctx, api, SQLITE_SOURCE_API_POINTER_NAME, destroy)
**
** where `api` points at a `sqlite_source_api` struct. A *consumer* extension
** (sqlite-xsv, sqlite-parquet, ...) evaluates `SELECT _http_api()`, pulls the
** pointer out with sqlite3_value_pointer(sqlite3_column_value(stmt, 0), NAME),
** copies the struct, calls `retain(ctx)`, and finalizes the statement. The
** copy stays valid until the consumer calls `release(ctx)`.
**
** Rules:
**  - `ctx` must be usable from any thread; consumers may call concurrently.
**  - All int-returning operations return SQLITE_SOURCE_RC_OK on success. On
**    failure they return non-zero and, if `errmsg` is non-NULL, set `*errmsg`
**    to a NUL-terminated string that the consumer must pass to `free_string`.
**    SQLITE_SOURCE_RC_CHANGED means the `if_match` precondition failed.
**  - `get` / `get_range` take an optional `if_match` ETag (NULL = none), as
**    returned by `head`, so multi-request reads can detect a changed object.
**  - Strings in `sqlite_source_meta` (etag, content_type) are NULL or
**    producer-allocated; free them with `free_string`.
**  - `get_range` buffers are freed with `free_buffer(ctx, buf, buf_len)`.
**  - Streams are freed with `stream->close(stream)`.
**  - `list` is reserved and must be NULL in ABI version 1.
**
** Layout check (64-bit): sizeof(sqlite_source_meta)   == 32
**                        sizeof(sqlite_source_stream) == 24
**                        sizeof(sqlite_source_api)    == 80
** The Rust mirror lives in sqlite-loadable-rs `src/source.rs`.
*/
#ifndef SQLITE_SOURCE_H
#define SQLITE_SOURCE_H

#include <stdint.h>

#define SQLITE_SOURCE_API_POINTER_NAME "sqlite-source-api-v1"
#define SQLITE_SOURCE_ABI_VERSION 1

#define SQLITE_SOURCE_RC_OK 0
#define SQLITE_SOURCE_RC_ERROR 1
#define SQLITE_SOURCE_RC_CHANGED 2

#define SQLITE_SOURCE_SIZE_UNKNOWN UINT64_MAX
#define SQLITE_SOURCE_LAST_MODIFIED_UNKNOWN ((int64_t)-1)

typedef struct sqlite_source_meta {
  uint64_t size;             /* SQLITE_SOURCE_SIZE_UNKNOWN if unknown */
  int64_t last_modified_ms;  /* unix epoch ms; SQLITE_SOURCE_LAST_MODIFIED_UNKNOWN if unknown */
  char *etag;                /* NULL or producer-allocated (free_string) */
  char *content_type;        /* NULL or producer-allocated (free_string) */
} sqlite_source_meta;

typedef struct sqlite_source_stream sqlite_source_stream;
struct sqlite_source_stream {
  void *ctx;
  /* returns bytes read; 0 at EOF; <0 on error with *errmsg set */
  int64_t (*read)(void *ctx, uint8_t *buf, uint64_t len, char **errmsg);
  /* frees the stream, its ctx, and the struct itself */
  void (*close)(sqlite_source_stream *stream);
};

typedef struct sqlite_source_api {
  uint32_t abi_version;   /* SQLITE_SOURCE_ABI_VERSION */
  uint32_t struct_size;   /* sizeof(sqlite_source_api) as built by the producer */
  void *ctx;
  void (*retain)(void *ctx);
  void (*release)(void *ctx);
  int (*head)(void *ctx, const char *url, sqlite_source_meta *out, char **errmsg);
  int (*get)(void *ctx, const char *url, const char *if_match,
             sqlite_source_stream **out, char **errmsg);
  int (*get_range)(void *ctx, const char *url, uint64_t start, uint64_t len,
                   const char *if_match, uint8_t **buf, uint64_t *buf_len, char **errmsg);
  void (*free_string)(void *ctx, char *s);
  void (*free_buffer)(void *ctx, uint8_t *p, uint64_t len);
  void *list;             /* reserved; NULL in ABI v1 */
} sqlite_source_api;

#endif /* SQLITE_SOURCE_H */
