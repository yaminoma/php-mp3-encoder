ifeq ($(shell uname),Darwin)
    EXT := dylib
else
    EXT := so
endif

all: target/debug/libmp3encoder.$(EXT)
	php -d ffi.enable=1 -d post_max_size=100M -d upload_max_filesize=100M -d memory_limit=8192M -d max_execution_time=300 -S localhost:8000 -t src

target/debug/libmp3encoder.$(EXT): src/lib.rs Cargo.toml
	cargo build --release

clean:
	rm -rf target
