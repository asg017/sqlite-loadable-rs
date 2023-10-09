build/test_auto: test_auto.c
	gcc $< \
		-Os \
		-I../../target/debug/examples/ \
		-L../../target/debug/examples/ \
		-L/usr/local/opt/sqlite/lib \
		-DSQLITE_CORE -Wl,-undefined,dynamic_lookup \
		-lsqlite3 -lhello \
		$(LDFLAGS) \
		-o $@

test_auto: build/test_auto
	./build/test_auto

build/test_manual: test_manual.c
	gcc $< \
		-Os \
		-I../../target/debug/examples/ \
		-L../../target/debug/examples/ \
		-L/usr/local/opt/sqlite/lib \
		-DSQLITE_CORE -Wl,-undefined,dynamic_lookup \
		-lsqlite3 -lhello \
		$(LDFLAGS) \
		-o $@

test_manual: build/test_manual
	./build/test_manual

test: test_auto test_manual


build/test_auto_go: test_auto.go
	CGO_LDFLAGS="-L../../target/debug/examples" \
	go build -o $@ $<

.PHONY: test
