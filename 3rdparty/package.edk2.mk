$(OUT)/3rdparty/edk2/edk2-aarch64-code.fd: $(SRCTREE)/3rdparty/edk2/edk2-aarch64-code.fd.bz2
	@echo "  BUNZIP2  $@"
	@mkdir -p $(@D)
	@bzip2 -dkc $< > $@
