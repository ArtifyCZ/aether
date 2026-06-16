rust-project.json: FORCE
	@echo "  RUSTC CMDS  $@"
	@cd $(SRCTREE) && BUILD_WORKSPACE_DIRECTORY=$(SRCTREE) bash tooling/sync_ide/discover_bazel_rust_project.sh "$(ARCH)"
