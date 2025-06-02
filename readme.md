# The Widow Programming Language

A modern programming language with Python-like simplicity and Rust-like safety.

- **File extension:** `.wd`
- **Implementation:** Written in Rust
- **Execution model:** Compiles to bytecode for the Widow VM with option for native executables
- **Type system:** Strong typing with automatic type inference
- **Memory management:** Automatic with borrow checker

## Design Philosophy

Widow combines the simplicity of Python and Go with the safety of Rust:

- **Simple syntax** with minimal ceremony
- **Strong typing** with automatic type inference
- **Memory safety** without manual memory management
- **Bytecode compilation** for portability and execution speed

## Core Language Features

### Variables

```widow/examples/variables.wd#L1-10
# Simple variable declaration with automatic type inference
x = 5
name = "Alice"

# Explicit type annotations (optional)
x:i32 = 5
name:String = "Alice"

# Constants
const PI = 3.14159
```

### Data Types

```widow/examples/types.wd#L1-20
# Integer types
i:i8 = 127        # 8-bit signed integer
i:i32 = 42        # 32-bit signed integer
i:i64 = 42        # 64-bit signed integer
i:i128 = 42       # 128-bit signed integer
i:iarch = 42      # Architecture-dependent sized integer

u:u8 = 255        # 8-bit unsigned integer
u:u32 = 42        # 32-bit unsigned integer
u:u64 = 42        # 64-bit unsigned integer
u:u128 = 42       # 128-bit unsigned integer
u:uarch = 42      # Architecture-dependent unsigned integer

# Floating point
f:f32 = 3.14      # 32-bit float
f:f64 = 3.14159   # 64-bit float
f:farch = 3.14    # Architecture-dependent float

# Other primitives
b = true          # Boolean
c = 'a'           # Character
s = "Hello"       # String
```

### Collections

```widow/examples/collections.wd#L1-8
# Arrays
numbers = [1, 2, 3, 4, 5]

# Maps (dictionaries)
scores = {"Alice": 95, "Bob": 87}

# Type-specific map (optional)
ids:hm<String, i32> = {"user1": 101, "user2": 102}
```

## Control Flow

```widow/examples/control_flow.wd#L1-20
# If statements
if x > 5:
    print("x is greater than 5")
elif x == 5:
    print("x equals 5")
else:
    print("x is less than 5")

# For loops with range
for i in 1..5:
    print(i)

# For loops with collections
for name in ["Alice", "Bob", "Charlie"]:
    print("Hello, " + name)
        
# Condition-based loop
for x > 0:
    x -= 1
```

### Switch/Case Statements

```widow/examples/switch_case.wd#L1-14
# Basic switch statement
switch day:
    case "Monday":
        print("Start of work week")
    case "Tuesday", "Wednesday", "Thursday":
        print("Mid-week")
    case "Friday":
        print("End of work week")
    case "Saturday", "Sunday":
        print("Weekend")
    default:
        print("Invalid day")
```

## Functions

```widow/examples/functions.wd#L1-15
# Simple function
func add(a, b):
    ret a + b

# Function with type annotations (optional)
func multiply(a:i32, b:i32) -> i32:
    ret a * b

# Multiple return values
func divide(a, b):
    if b == 0:
        ret 0, "Division by zero"
    ret a / b, nil

quotient, err = divide(10, 2)
```

## Error Handling

```widow/examples/error_handling.wd#L1-13
# Return-based error handling (similar to Go)
func read_file(path):
    if !file_exists(path):
        ret nil, "File not found"
    content = file_read(path)
    ret content, nil

# Usage
content, err = read_file("config.txt")
if err != nil:
    print("Error: " + err)
else:
    print("File content: " + content)
```

## Structs

```widow/examples/structs.wd#L1-9
# Struct definition
struct Person:
    name:String
    age:i32

# Struct instantiation
alice = Person{
    name: "Alice",
    age: 30
}
```

### Implementations

```widow/examples/implementations.wd#L1-12
# Implementation for a struct
impl Person:
    # Constructor
    func new(name, age) -> Person:
        ret Person{name: name, age: age}
    
    # Method with self reference
    func greet(self):
        print("Hello, my name is " + self.name)

# Usage
bob = Person.new("Bob", 25)
bob.greet()  # Prints: Hello, my name is Bob
```

## Memory Management

Widow leverages automatic memory management with a borrow checker system:

- Memory is automatically managed without manual allocation/deallocation
- The borrow checker ensures memory safety at runtime
- No garbage collection pauses

## Compilation & Execution

Widow compiles to bytecode that runs on the Widow Virtual Machine:

```
# Run a Widow program
widow hello.wd

# Compile explicitly to bytecode
widow compile hello.wd    # Creates hello.wdb (bytecode)
widow run hello.wdb       # Run the bytecode
```

## Future Features

The following features are planned for future versions:

### Advanced Type System
- Traits/Interfaces
- Generics
- Advanced pattern matching

### Concurrency
- Goroutine-like concurrent tasks
- Channel-based communication
- Thread safety guarantees

### Native Compilation
- Compilation to native executables
- Cross-compilation support
- Optimization levels

### Standard Library
- IO operations and file handling
- Network communications
- Data structures and algorithms
- Text processing

## Getting Started

```widow/examples/hello.wd#L1-3
func main():
    print("Hello, Widow!")
```

## Implementation Plan

1. Basic syntax and parser
2. Type system implementation
3. Memory management with borrow checker
4. Bytecode generation
5. VM implementation
6. Standard library basics

## Next Steps

- [Language Specification](https://widow-lang.org/spec)
- [Compiler Documentation](https://widow-lang.org/compiler)
- [VM Documentation](https://widow-lang.org/vm)