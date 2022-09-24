PREFIX = $(DESTDIR)/usr/local
BINDIR = $(PREFIX)/bin
MANDIR = $(PREFIX)/share/man

all: target/debug/fmm docs

target:
	mkdir target

target/debug/fmm: src/*.rs target
	cargo build

release:
	cargo build --release

test:
	cargo test

docs: target/man/fmm.1 target/man/fmm.5

target/man:
	mkdir -p target/man

target/man/fmm.1: man/fmm.1.scd target/man
	scdoc < $< > $@

target/man/fmm.5: man/fmm.5.scd target/man
	scdoc < $< > $@

clean:
	cargo clean

install:
	install -d \
		$(BINDIR) \
		$(MANDIR)/man1/ \
		$(MANDIR)/man5/
	install -pm 0755 target/release/fmm $(BINDIR)/
	install -pm 0644 target/man/fmm.1 $(MANDIR)/man1/
	install -pm 0644 target/man/fmm.5 $(MANDIR)/man5/

uninstall:
	rm -f \
		$(BINDIR)/fmm \
		$(MANDIR)/man1/fmm.1 \
		$(MANDIR)/man5/fmm.5

.PHONY: all release test docs clean install uninstall
