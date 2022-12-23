PREFIX = $(DESTDIR)/usr/local
BINDIR = $(PREFIX)/bin
MANDIR = $(PREFIX)/share/man

all: fmm docs

fmm: *.go
	go build

test:
	@if [ -d TEST ]; then echo "rm -rf TEST"; rm -rf TEST; fi
	cp -rf testfiles TEST
	@go test
	rm -rf TEST

docs: man/fmm.1 man/fmm.5

man/fmm.1: man/fmm.1.scd
	scdoc < $< > $@

man/fmm.5: man/fmm.5.scd
	scdoc < $< > $@

clean:
	go clean
	rm -f man/fmm.1
	rm -f man/fmm.5

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
