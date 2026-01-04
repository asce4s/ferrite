.PHONY: build install uninstall

# Installation paths
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
TMPFILESDIR ?= /etc/tmpfiles.d
TMPFILESCONF ?= ferrite.conf

# Binary name
BINARY = ferrite
TARGET = target/release/$(BINARY)

build:
	cargo build --release

install:
	@test -f $(TARGET) || (echo "Error: Binary not found at $(TARGET)" && \
		echo "Please build the project first with: make build" && \
		echo "Then run: sudo make install" && exit 1)
	install -Dm755 $(TARGET) $(DESTDIR)$(BINDIR)/$(BINARY)
	install -Dm644 systemd-tmpfiles.conf $(DESTDIR)$(TMPFILESDIR)/$(TMPFILESCONF)
	@echo "Installation complete!"
	@echo "Don't forget to run: sudo systemd-tmpfiles --create /etc/tmpfiles.d/$(TMPFILESCONF)"

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(BINARY)
	rm -f $(DESTDIR)$(TMPFILESDIR)/$(TMPFILESCONF)
	@echo "Uninstallation complete!"

