from syscall_parser import VALID_SYSCALL_VALUE_TYPES, Constant, SyscallDefinition, SyscallError, SyscallParser, Type
import sys

def main():
    # Retrieve the toml path from the argv
    if len(sys.argv) != 3:
        print("Usage: python syscall_c_stubs.py <path_to_syscalls.toml> <path_to_errors.toml>", file=sys.stderr)
        sys.exit(1)
        pass
    toml_path = sys.argv[1]
    errors_path = sys.argv[2]
    parser = SyscallParser([toml_path, errors_path])
    generated_output = parser.generate(adapter=CompatAdapter())

    print("#pragma once\n")
    for syscall in generated_output.syscalls:
        print(syscall)
        print()
        pass

    for constant in generated_output.constants:
        print(constant)
        pass

    print("Generated syscalls for legacy compatibility successfully!", file=sys.stderr)
    pass

class CompatAdapter:
    def translate_type(self, type_obj: Type) -> str:
        TYPE_TRANSLATION = {
            "unit": "void",
            "uint8": "uint8_t",
            "uint32": "uint32_t",
            "uint64": "uint64_t",
            "int32": "int",
            "uptr": "uintptr_t",
            "usize": "size_t",
            "const uint8*": "const void*",
        }

        for valid_type in VALID_SYSCALL_VALUE_TYPES:
            assert valid_type in TYPE_TRANSLATION, f"Missing translation for type: {valid_type}"
            pass

        return TYPE_TRANSLATION[type_obj.name]

    def render_syscall(self, syscall: SyscallDefinition) -> str:
        number = hex(syscall.number)
        return_type = self.translate_type(syscall.return_type)

        args_count = len(syscall.args)
        fn_args = ", ".join([f"{self.translate_type(arg.type)} {arg.name}" for arg in syscall.args])
        if return_type != "void":
            if args_count > 0:
                fn_args = fn_args + ", "
            fn_args = fn_args + f"{return_type} *out"
            pass
        else:
            fn_args = fn_args if args_count > 0 else "void"
            pass

        body_args = ", ".join([f"(uint64_t) {arg.name}" for arg in syscall.args])
        if args_count == 0:
            body_args = "0"
            pass

        if return_type != "void":
            fn_body = f"""
sys_result_t result = __arch_syscall({number}, {body_args});
if (result.err_code == SYS_SUCCESS && out != NULL) *out = result.value;
return result.err_code;
            """
            pass
        else:
            fn_body = f"""
return __arch_syscall({number}, {body_args}).err_code;
            """
            pass

        return f"""
__attribute__((noinline))
static sys_err_t sys_{syscall.name}({fn_args}) {{
    {fn_body}
}}
        """

    def render_constant(self, constant: Constant) -> str:
        name = f"SYS_{constant.name.upper()}"
        value = hex(constant.value)
        return f"#define {name} {value}"

    def render_error(self, error: SyscallError) -> str:
        # TODO: also generate something for the error handling and so
        return ""

if __name__ == "__main__":
    main()
    sys.exit(0)
