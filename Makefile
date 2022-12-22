PREFIX = $(DESTDIR)/usr/local
BINDIR = $(PREFIX)/bin
MANDIR = $(PREFIX)/share/man

all: fmm docs

fmm: *.go
	go build

test:
	go test

docs: fmm.1 fmm.5

fmm.1: fmm.1.scd
	scdoc < $< > $@

fmm.5: fmm.5.scd
	scdoc < $< > $@

clean:
	go clean
	rm -f fmm.1
	rm -f fmm.5

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

.PHONY: test docs clean install uninstall
