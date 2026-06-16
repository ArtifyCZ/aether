$(OUT)/image/pkg/aether.tar: FORCE | $(OUT)/image/pkg/
	@echo "  BUILD    $@"
	@cd $(SRCTREE) && bazel build //image/pkg --config=$(ARCH)
	@rm -f $@
	@cp $(SRCTREE)/bazel-bin/image/pkg/aether.tar $@
