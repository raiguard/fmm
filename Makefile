PREFIX = $(DESTDIR)/usr/local
BINDIR = $(PREFIX)/bin

FILES = $(shell find . -type f -name "*.go")

all: fmm

fmm: $(FILES)
	go build

test:
	@if [ -d TEST ]; then echo "rm -rf TEST"; rm -rf TEST; fi
	cp -rf mock TEST
	go test ./...
	rm -rf TEST

clean:
	go clean

install:
	install -Dpm 0755 fmm $(BINDIR)/fmm

uninstall:
	rm -f $(BINDIR)/fmm

.PHONY: test clean install uninstall
