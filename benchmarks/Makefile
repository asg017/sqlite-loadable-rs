
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


SHELL=/bin/bash

scalar-yo.png: ./scalar/results-yo.json Makefile
	observable-prerender @asg017/hyperfine-benchmark-viz1 chart  \
		--redefine=width:900 \
		--width 900 \
		--redefine-file=data:json:$< \
		--redefine-file=highlight:json:<(echo '["./yo_rs.sh 1e6"]') \
		'--redefine=compareTo:./yo_rs.sh 1e6' \
		'--redefine=title:Run yo(), a static deterministic function, 1 million times' \
		'--redefine=footer:ran on a 2019 Macbook, i5 processing and 8GB RAM' \
		-o $@


scalar-add.png: ./scalar/results-add.json Makefile
	observable-prerender @asg017/hyperfine-benchmark-viz1 chart  \
		--redefine=width:900 \
		--width 900 \
		--redefine-file=data:json:$< \
		--redefine-file=highlight:json:<(echo '["./add_rs.sh 1e6"]') \
		'--redefine=compareTo:./add_rs.sh 1e6' \
		'--redefine=title:Run add(a, b) 1 million times on different inputs' \
		'--redefine=footer:ran on a 2019 Macbook, i5 processing and 8GB RAM' \
		-o $@

scalar-surround.png: ./scalar/results-surround.json Makefile
	observable-prerender @asg017/hyperfine-benchmark-viz1 chart  \
		--redefine=width:900 \
		--width 900 \
		--redefine-file=data:json:$< \
		--redefine-file=highlight:json:<(echo '["./surround_rs.sh"]') \
		'--redefine=compareTo:./surround_rs.sh' \
		'--redefine=title:run surround(x) on every word in /usr/share/dict/words ' \
		'--redefine=footer:ran on a 2019 Macbook, i5 processing and 8GB RAM' \
		-o $@

series.png: ./series/results.json Makefile
	observable-prerender @asg017/hyperfine-benchmark-viz1 chart  \
		--redefine=width:900 \
		--width 900 \
		--redefine-file=data:json:$< \
		--redefine-file=highlight:json:<(echo '["./series_rs.sh 1e6"]') \
		'--redefine=compareTo:./series_rs.sh 1e6' \
		'--redefine=title:Run generate_series() and return 1 million rows' \
		'--redefine=footer:ran on a 2019 Macbook, i5 processing and 8GB RAM' \
		-o $@

scalar_pngs: scalar-add.png scalar-surround.png scalar-yo.png

clean:
	rm target/*
	rm *.png

.PHONY: all clean series scalar

