BINDIR = bin
MANDIR = share/man
PREFIX = /usr/local

all: build docs

build:
	cargo build --release

docs: doc/fmm.1 doc/fmm.5

doc/fmm.1: doc/fmm.1.scd
	scdoc < $< > $@

doc/fmm.5: doc/fmm.5.scd
	scdoc < $< > $@

clean:
	cargo clean
	rm -f doc/fmm.1 doc/fmm.5

install:
	install -d \
		$(DESTDIR)$(PREFIX)/$(BINDIR) \
		$(DESTDIR)$(PREFIX)/$(MANDIR)/man1/ \
		$(DESTDIR)$(PREFIX)/$(MANDIR)/man5/
	install -pm 0755 target/release/fmm $(DESTDIR)$(PREFIX)/$(BINDIR)/
	install -pm 0644 doc/fmm.1 $(DESTDIR)$(PREFIX)/$(MANDIR)/man1/
	install -pm 0644 doc/fmm.5 $(DESTDIR)$(PREFIX)/$(MANDIR)/man5/

uninstall:
	rm -f \
		$(DESTDIR)$(PREFIX)/$(BINDIR)/fmm \
		$(DESTDIR)$(PREFIX)/$(MANDIR)/man1/fmm.1 \
		$(DESTDIR)$(PREFIX)/$(MANDIR)/man5/fmm.5

.PHONY: all build docs clean install
