from syscall_parser import VALID_SYSCALL_VALUE_TYPES, Constant, SyscallDefinition, SyscallParser, Type
import sys

def main():
    # Retrieve the toml path from the argv
    if len(sys.argv) != 2:
        print("Usage: python compat_generator.py <path_to_syscalls.toml>", file=sys.stderr)
        sys.exit(1)
        pass
    toml_path = sys.argv[1]
    parser = SyscallParser([toml_path])
    generated_output = parser.generate(adapter=CompatAdapter())

    print("#pragma once\n")
    print("#define SYSCALLS_LIST(X) \\")
    for syscall in generated_output.syscalls:
        print(syscall)
        pass
    print("    /* End of generated syscalls */\n")

    for constant in generated_output.constants:
        print(constant)
        pass

    print("Generated syscalls for legacy compatibility successfully!", file=sys.stderr)
    pass

class CompatAdapter:
    def translate_type(self, type_obj: Type) -> str:
        # We handle return types slightly differently to match legacy uintptr_t/void
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
        return_type = self.translate_type(syscall.return_type)

        args_count = len(syscall.args)

        # Render argument list: type1, name1, type2, name2...
        # Legacy X macro: X(name, NAME, num, ret, cnt, args...)
        args_rendered = ""
        if args_count > 0:
            raw_args = []
            for arg in syscall.args:
                raw_args.append(self.translate_type(arg.type))
                raw_args.append(arg.name)
            args_rendered = ", " + ", ".join(raw_args)

        return f"    X({syscall.id}, {syscall.id.upper()}, {hex(syscall.number)}, {return_type}, {args_count}{args_rendered}) \\"
    
    def render_constant(self, constant: Constant) -> str:
        name = f"SYS_{constant.name.upper()}"
        value = hex(constant.value)
        return f"#define {name} {value}"

if __name__ == "__main__":
    main()
    sys.exit(0)
