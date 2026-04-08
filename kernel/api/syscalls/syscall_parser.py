import tomllib
from dataclasses import dataclass

VALID_SYSCALL_VALUE_TYPES = [
    # The unit type represents the absence of a value, similar to void in C or () in Rust.
    # Used for syscalls that do not return any meaningful value.
    "unit",
    "uint8", 
    "uint32",
    "uint64",
    # A pointer-sized integer type, similar to uintptr_t in C.
    "uptr",
    "usize",
    "int32",
    # Represents a pointer to constant data, often used for input buffers.
    "const uint8*", 
]

@dataclass
class Type:
    def __init__(self, name: str):
        assert name in VALID_SYSCALL_VALUE_TYPES, f"Invalid type: {name}. Valid types are: {VALID_SYSCALL_VALUE_TYPES}"
        self.name = name
    pass

@dataclass
class SyscallDefinition:
    def __init__(self, id: str, data: dict):
        assert id != "", "Syscall identifier cannot be empty"
        self.id = id
        self.number = int(data["number"])
        assert self.number >= 0, "Syscall number must be a non-negative integer"
        assert self.number <= 255, "Probably a bug: Syscall number greater than 255"
        self.name = str(data["name"] if "name" in data else id)
        self.args = [SyscallArgument(arg) for arg in data["args"]]
        self.return_type = Type(str(data["return_type"]))
        assert len(self.args) <= 5, "Syscalls cannot have more than 5 arguments"
    pass

@dataclass
class SyscallArgument:
    def __init__(self, data: dict):
        self.name = str(data["name"])
        self.type = Type(str(data["type"]))
    pass

@dataclass
class Constant:
    def __init__(self, name: str, value: int):
        assert name != "", "Constant name cannot be empty"
        assert isinstance(value, int), "Constant value must be an integer"
        self.name = name
        self.value = value
    pass

@dataclass
class GeneratedOutput:
    def __init__(self, syscalls: list[SyscallDefinition], constants: list[Constant]):
        assert len(syscalls) > 0, "Empty list of syscalls is probably a bug, and therefore failed to generate any code"
        self.syscalls = syscalls
        self.constants = constants
    pass

class SyscallParser:
    def __init__(self, toml_paths: list[str]):
        self.syscalls = {}
        self.constants = {}
        for toml_path in toml_paths:
            with open(toml_path, "rb") as f:
                definitions = tomllib.load(f)
                self.syscalls |= definitions["syscalls"]
                self.constants |= definitions["constants"]
                pass
            pass
        pass

    def generate(self, adapter) -> str:
        generated_syscalls = []
        generated_constants = []

        # Implementation for generating syscall code
        for id, syscall in self.syscalls.items():
            syscall_def = SyscallDefinition(id, syscall)
            generated_syscall = adapter.render_syscall(syscall_def)
            generated_syscalls.append(generated_syscall)
            pass

        for name, value in self.constants.items():
            constant = Constant(name, value)
            generated_constant = adapter.render_constant(constant)
            generated_constants.append(generated_constant)
            pass

        return GeneratedOutput(generated_syscalls, generated_constants)
    pass
