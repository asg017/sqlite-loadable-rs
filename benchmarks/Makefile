
TARGET_SERIES_C=target/series_c.dylib
TARGET_SERIES_GO=target/series_go.dylib
TARGET_SERIES_RS=target/series_rs.dylib

TARGET_SCALAR_C=target/scalar_c.dylib
TARGET_SCALAR_GO=target/scalar_go.dylib
TARGET_SCALAR_RS=target/scalar_rs.dylib

# SERIES
$(TARGET_SERIES_C): series/series.c
	gcc -O3 -I ../sqlite3ext-sys/sqlite3/ $< -fPIC -shared -o $@

$(TARGET_SERIES_GO): series/series.go
	go build -buildmode=c-shared -o $@ $<

$(TARGET_SERIES_RS): ../examples/series.rs
	cargo build --manifest-path=../Cargo.toml --example series --release
	cp ../target/release/examples/libseries.dylib $@

# SCALAR
$(TARGET_SCALAR_C): scalar/scalar.c
	gcc -O3 -I ../sqlite3ext-sys/sqlite3/ $< -fPIC -shared -o $@

$(TARGET_SCALAR_GO): scalar/scalar.go
	go build -buildmode=c-shared -o $@ $<

$(TARGET_SCALAR_RS): ../examples/scalar.rs
	cargo build --manifest-path=../Cargo.toml --example scalar --release
	cp ../target/release/examples/libscalar.dylib $@



series: $(TARGET_SERIES_GO) $(TARGET_SERIES_C) $(TARGET_SERIES_RS)
scalar: $(TARGET_SCALAR_GO) $(TARGET_SCALAR_C) $(TARGET_SCALAR_RS)

all: series scalar

clean:
	rm target/*

.PHONY: all clean series scalar

