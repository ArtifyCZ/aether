all: $(OUT)/syscall_c_stubs.h

# syscall_c_stubs.py uses tomllib (Python >= 3.11); macOS system python3 is 3.9.
PYTHON ?= $(shell for p in python3.14 python3.13 python3.12 python3.11 python3; do \
	command -v $$p >/dev/null 2>&1 && \
	$$p -c 'import sys; sys.exit(0 if sys.version_info >= (3,11) else 1)' 2>/dev/null && \
	{ echo $$p; break; }; \
done)

pkgLibsLibsyscallSyscallsToml := $(SRCTREE)/kernel/api/syscalls/syscalls.toml
pkgLibsLibsyscallErrorsToml   := $(SRCTREE)/kernel/api/syscalls/errors.toml
pkgLibsLibsyscallParser       := $(SRCTREE)/kernel/api/syscalls/syscall_parser.py
pkgLibsLibsyscallStubsScript  := $(SRCTREE)/libs/libsyscall/syscall_c_stubs.py

$(OUT)/syscall_c_stubs.h: \
		$(pkgLibsLibsyscallStubsScript) \
		$(pkgLibsLibsyscallParser) \
		$(pkgLibsLibsyscallSyscallsToml) \
		$(pkgLibsLibsyscallErrorsToml)
	@echo "  GEN      $@"
	@[ -n "$(PYTHON)" ] || { echo "no Python >= 3.11 found on PATH (needed for tomllib); set PYTHON to override" >&2; exit 1; }
	PYTHONPATH=$(SRCTREE)/kernel/api/syscalls $(PYTHON) $(pkgLibsLibsyscallStubsScript) $(pkgLibsLibsyscallSyscallsToml) $(pkgLibsLibsyscallErrorsToml) > $@.tmp
	mv $@.tmp $@
