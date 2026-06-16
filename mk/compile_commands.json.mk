compile_commands.json: FORCE
	@echo "  CC CMDS  $@"
	@cd $(SRCTREE) && python3 tooling/sync_ide/discover_bazel_c_compile_commands.py --config="$(ARCH)"
