.PHONY: all run clean install

all: libdisk.a
	cargo build --release

run: libdisk.a
	cargo run

libdisk.a: disk.o
	cc -r -o libdisk.a disk.o

disk.o: src/disk.c
	cc -c src/disk.c

clean:
	rm -f libdisk.a disk.o
	cargo clean

install:
	cargo install --path .