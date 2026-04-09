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
    def __init__(self, name: str, data: dict):
        assert name != "", "Constant name cannot be empty"
        POSSIBLE_TYPES = [Type("uint8"), Type("uint32"), Type("uint64")]
        assert len(data) == 1, f"Constant value for {name} must have exactly one entry with the type as the key and the integer value as the value, but got: {data}"
        type_name, value = next(iter(data.items()))
        assert type_name in [t.name for t in POSSIBLE_TYPES], f"Constant value {name} has unsupported or invalid type: {type_name}"
        type = Type(type_name)
        assert isinstance(value, int), "Constant value must be an integer"
        self.name = name
        self.value = value
        self.type = type
    pass

@dataclass
class SyscallError:
    def __init__(self, id: str, data: dict):
        assert id != "", "Error identifier cannot be empty"
        self.id = id
        self.code = int(data["code"])
        assert self.code >= 0, "Error code must be a non-negative integer"
        assert self.code <= 255, "Probably a bug: Error code greater than 255"
        self.name = str(data["name"] if "name" in data else id)
        self.message = str(data["message"])
        assert self.message != "", "Error message cannot be empty"
    pass

@dataclass
class GeneratedOutput:
    def __init__(self, syscalls: list[SyscallDefinition], constants: list[Constant], errors: list[SyscallError]):
        assert len(syscalls) > 0, "Empty list of syscalls is probably a bug, and therefore failed to generate any code"
        assert len(constants) > 0, "Empty list of constants is probably a bug, and therefore failed to generate any code"
        assert len(errors) > 0, "Empty list of errors is probably a bug, and therefore failed to generate any code"
        self.syscalls = syscalls
        self.constants = constants
        self.errors = errors
    pass

class SyscallParser:
    def __init__(self, toml_paths: list[str]):
        self.syscalls = {}
        self.constants = {}
        self.errors = {}
        for toml_path in toml_paths:
            with open(toml_path, "rb") as f:
                definitions = tomllib.load(f)
                self.syscalls |= definitions.get("syscalls", {})
                self.constants |= definitions.get("constants", {})
                self.errors |= definitions.get("errors", {})
                pass
            pass
        pass

    def generate(self, adapter) -> GeneratedOutput:
        generated_syscalls = []
        generated_constants = []
        generated_errors = []

        for id, syscall in self.syscalls.items():
            syscall_def = SyscallDefinition(id, syscall)
            generated_syscall = adapter.render_syscall(syscall_def)
            generated_syscalls.append(generated_syscall)
            pass

        for name, value in self.constants.items():
            assert isinstance(value, dict), f"Constant value for {name} must have exactly one entry with the type as the key and the integer value as the value, but got: {value}"
            constant = Constant(name, value)
            generated_constant = adapter.render_constant(constant)
            generated_constants.append(generated_constant)
            pass

        for id, error in self.errors.items():
            error_def = SyscallError(id, error)
            generated_error = adapter.render_error(error_def)
            generated_errors.append(generated_error)
            pass

        return GeneratedOutput(generated_syscalls, generated_constants, generated_errors)
    pass
