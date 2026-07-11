PACKAGES += libs/libsyscall

PHONY += pkg/libs/libsyscall/install
pkg/libs/libsyscall/install: \
	$(SYSROOT)/usr/include/syscall/syscalls.h \
	$(SYSROOT)/usr/include/syscall/syscall_c_stubs.h

$(SYSROOT)/usr/include/syscall/syscalls.h: $(SRCTREE)/libs/libsyscall/syscalls.h | $(SYSROOT)/usr/include/syscall/
	@echo "  INSTALL  $@"
	rm -f $@
	cp $< $@

pkgLibsLibsyscallOut := $(PKGS)/libs/libsyscall
pkgLibsLibsyscallOut := $(abspath $(pkgLibsLibsyscallOut))

$(PKGS)/libs/libsyscall/syscall_c_stubs.h: FORCE | $(PKGS)/libs/libsyscall/
	@echo "  MAKE  pkg/libs/libsyscall"
	$(MAKE) -C $(SRCTREE)/libs/libsyscall OUT=$(pkgLibsLibsyscallOut) -f build.mk

$(SYSROOT)/usr/include/syscall/syscall_c_stubs.h: $(PKGS)/libs/libsyscall/syscall_c_stubs.h | $(SYSROOT)/usr/include/syscall/
	@echo "  INSTALL  $@"
	rm -f $@
	cp $< $@
