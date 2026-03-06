// Dummy C file to satisfy Bazel's requirement for a static library
// This file exists only to make the cc_library produce a static archive
// so that rust_bindgen_library can work properly.

void __syscalls_dummy(void) {
    // This function intentionally does nothing
}