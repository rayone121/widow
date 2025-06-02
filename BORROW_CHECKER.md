# Widow Automatic Borrow Checker Implementation

## Overview

The Widow language implements an automatic borrow checking system that leverages Rust's built-in borrow checker to enforce memory safety at runtime. Unlike Rust, Widow doesn't require explicit lifetime annotations or ownership declarations in the source code, making the language more approachable while still maintaining memory safety.

## Core Concepts

### Memory Safety Guarantees

Widow's borrow checker ensures:
1. No use-after-free errors
2. No data races (concurrent read/write or write/write conflicts)
3. No dangling references
4. Safe mutation of data

### Implementation Strategy

Widow uses a hybrid approach:
1. **Runtime representation**: Tracking borrow states and variable ownership at runtime
2. **Compile-time safety**: Leveraging Rust's built-in borrow checker for the underlying implementation

## Key Components

### Value Representation

Values in Widow are represented using Rust's `RefCell` and `Rc` types:
- `RefCell`: Provides interior mutability with runtime borrow checking
- `Rc`: Enables shared ownership of values

This allows multiple language-level references to the same data while maintaining Rust's safety guarantees.

### Borrow States

The VM tracks three possible borrow states for each variable:
1. **None**: No active borrows
2. **Shared(n)**: n active immutable borrows
3. **Exclusive**: One active mutable borrow

### Runtime Borrow Checking

When a variable is accessed, the VM performs these checks:
- **Read access**: Ensure no exclusive borrow exists
- **Write access**: Ensure no borrows of any kind exist
- **Mutable borrow**: Ensure no other borrows exist
- **Immutable borrow**: Ensure no exclusive borrow exists

## Bytecode Instructions

The following bytecode instructions handle borrowing:

- `BorrowShared`: Create a shared borrow of a variable
- `BorrowMut`: Create a mutable borrow of a variable
- `ReleaseBorrow`: Release a previously created borrow
- `DefineGlobal`: Define a new variable in the current scope
- `PushScope`: Create a new scope level
- `PopScope`: Exit the current scope (automatically releases borrows)

## Variable Lifecycle

### 1. Declaration

```
x = 5  # Implicitly immutable
```

Bytecode:
- `Constant 0` (push 5 onto stack)
- `DefineGlobal "x"` (define variable x with value 5)

### 2. Immutable Borrow

```
y = x  # Immutable borrow of x
```

Bytecode:
- `BorrowShared "x"` (create shared borrow, push value to stack)
- `SetGlobal "y"` (assign the value to y)
- `ReleaseBorrow "x"` (release the borrow)

### 3. Mutable Access

```
x = x + 1  # Requires temporary immutable borrow for read, then assignment
```

Bytecode:
- `BorrowShared "x"` (read x's value)
- `Constant 1` (push 1 onto stack)
- `Add` (add the values)
- `SetGlobal "x"` (assign the result back to x)
- `ReleaseBorrow "x"` (release the borrow)

### 4. Scope Management

```
{
    temp = x  # Borrow in nested scope
}  # Borrow automatically released at scope end
```

Bytecode:
- `PushScope` (create new scope)
- `BorrowShared "x"` (create shared borrow)
- `SetGlobal "temp"` (define in current scope)
- `PopScope` (exit scope, releasing all borrows in that scope)

## Error Cases

Widow reports clear runtime errors when borrow rules are violated:

1. **Mutably borrowing when shared borrows exist**:
   ```
   let reader1 = &x
   let reader2 = &x   # OK: Multiple shared borrows allowed
   let writer = &mut x  # ERROR: Cannot mutably borrow when shared borrows exist
   ```

2. **Borrowing after mutation**:
   ```
   let writer = &mut x
   let reader = &x  # ERROR: Cannot immutably borrow when mutable borrow exists
   ```

3. **Use after move**:
   ```
   y = move x
   z = x  # ERROR: Value moved, cannot use x anymore
   ```

## Implementation Benefits

1. **User-friendly**: No explicit lifetime or ownership annotations in code
2. **Safe**: Memory safety guarantees without garbage collection pauses
3. **Deterministic**: Predictable resource cleanup
4. **Efficient**: No GC overhead or stop-the-world pauses

## Future Enhancements

- Static analysis to reduce runtime borrow checking overhead
- Ownership transfer optimizations
- Improved error messages with source code location