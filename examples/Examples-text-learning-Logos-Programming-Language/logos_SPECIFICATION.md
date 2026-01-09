# Logos Language Specification

## Overview

Logos is a modern, safe, multi-paradigm programming language that builds upon the the idea of mult macros lang programming, concept with significant enhancements. This specification defines the syntax, semantics, and core features of the Logos programming language.

## 1. Lexical Structure

### 1.1 Comments
logos
// Single-line comment
/* Multi-line
   comment */
/// Documentation comment


### 1.2 Identifiers
Identifiers start with a letter or underscore, followed by letters, digits, or underscores:
- variableName
- _private_field
- CONSTANT_VALUE

### 1.3 Keywords
Reserved words that cannot be used as identifiers:

as, async, await, break, case, catch, class, const, continue, def, elif, else, enum, except, 
extern, false, final, fn, for, if, impl, import, in, interface, let, loop, match, mod, mut, 
nil, override, package, pub, return, self, static, struct, super, trait, true, try, type, 
val, var, while, with, yield, actor, spawn, send, receive, region, linear, effect, perform


### 1.4 Literals

#### 1.4.1 Integer Literals

42          // Decimal
0xFF        // Hexadecimal
0o755       // Octal
0b1010      // Binary
1_000_000   // Underscores for readability


#### 1.4.2 Float Literals

3.14
1.0e10
1.5E-5


#### 1.4.3 String Literals

"Hello, World!"
Hello, World!  // Alternative syntax
"""Multiline
string"""


#### 1.4.4 Boolean Literals

true
false


## 2. Types

### 2.1 Primitive Types
- Int: Arbitrary precision integers
- Float: 64-bit floating point numbers
- Bool: Boolean values
- Char: Unicode characters
- String: UTF-8 encoded strings
- Unit: The type with a single value ()

### 2.2 Composite Types

#### 2.2.1 Arrays

[Int]           // Array of integers
[String]       // Array of strings
[T]            // Generic array


#### 2.2.2 Tuples

(Int, String)           // Pair
(Int, String, Bool)     // Triple
(T, U, V)              // Generic tuple


#### 2.2.3 Records

Person{name: String, age: Int}
Point{x: Float, y: Float}


#### 2.2.4 Maps

Map<String, Int>
Map<K, V>


#### 2.2.5 Option and Result Types

Option<T> = Some(T) | None
Result<T, E> = Ok(T) | Err(E)


### 2.3 User-Defined Types

#### 2.3.1 Enums
logos
enum Color {
    Red,
    Green,
    Blue
}

enum Result<T, E> {
    Ok(T),
    Err(E)
}


#### 2.3.2 Structs
logos
struct Point {
    x: Float,
    y: Float
}

struct Person {
    name: String,
    age: Int
}


#### 2.3.3 Classes
logos
class Animal {
    name: String

    fn init(name: String) {
        this.name = name
    }

    fn speak(self) -> String {
        return "Some sound"
    }
}


## 3. Expressions

### 3.1 Primary Expressions
- Literals
- Variables
- Parenthesized expressions
- Function calls
- Method calls
- Field access

### 3.2 Operators

#### 3.2.1 Arithmetic Operators

+   // Addition
-   // Subtraction
*   // Multiplication
/   // Division
%   // Modulo
^   // Exponentiation
**  // Alternative exponentiation


#### 3.2.2 Comparison Operators

==  // Equal
!=  // Not equal
<   // Less than
>   // Greater than
<=  // Less than or equal
>=  // Greater than or equal
<=> // Spaceship operator (returns -1, 0, or 1)


#### 3.2.3 Logical Operators

&&  // Logical AND
||  // Logical OR
!   // Logical NOT


#### 3.2.4 Bitwise Operators

&   // Bitwise AND
|   // Bitwise OR
^   // Bitwise XOR
<<  // Left shift
>>  // Right shift
~   // Bitwise NOT


#### 3.2.5 Assignment Operators

=    // Assignment
+=   // Add and assign
-=   // Subtract and assign
*=   // Multiply and assign
/=   // Divide and assign
%=   // Modulo and assign
^=   // Exponentiate and assign
**=  // Exponentiate and assign


#### 3.2.6 Pipeline Operators

|>   // Forward pipeline
<|   // Backward pipeline


### 3.3 Function Expressions
logos
|x| x + 1                    // Simple lambda
|x, y| x * y                 // Multi-parameter lambda
|| {                         // Block lambda
    let result = compute()
    return result
}


## 4. Statements

### 4.1 Variable Declarations
logos
let x = 42                    // Immutable binding
mut y = 0                     // Mutable binding
const PI = 3.14159           // Constant
let [a, b, c] = [1, 2, 3]   // Destructuring
let {name, age} = person      // Record destructuring


### 4.2 Assignment Statements
logos
x = 5
y += 10
obj.field = value


### 4.3 Control Flow Statements

#### 4.3.1 Conditional Statements
logos
if condition {
    // then block
} elif other_condition {
    // elif block
} else {
    // else block
}


#### 4.3.2 Loop Statements
logos
// While loop
while condition {
    // loop body
}

// For loop
for item in collection {
    // loop body
}

// For with index
for i, item in enumerate(collection) {
    // loop body
}

// Infinite loop
loop {
    // loop body
    if exit_condition {
        break
    }
}

// Loop with return value
let result = loop {
    if condition {
        break value
    }
}


#### 4.3.3 Match Expressions
logos
match value {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression  // Wildcard
}

// Match with guards
match value {
    x if x > 0 => "positive",
    x if x < 0 => "negative",
    _ => "zero"
}


#### 4.3.4 Exception Handling
logos
try {
    // risky code
    risky_operation()
} catch SpecificError(error) {
    // handle specific error
    handle_specific_error(error)
} except error: GeneralError {
    // handle general error
    handle_general_error(error)
} finally {
    // cleanup code
    cleanup_resources()
}


### 4.4 Function Definitions
logos
// Basic function
fn function_name(param: Type) -> ReturnType {
    // function body
    return value
}

// Generic function
fn identity<T>(value: T) -> T {
    return value
}

// Async function
async fn async_function(param: Type) -> ReturnType {
    let result = await some_async_operation()
    return result
}

// Higher-order function
fn apply<T, U>(f: fn(T) -> U, x: T) -> U {
    return f(x)
}


### 4.5 Class Definitions
logos
class ClassName {
    field: Type
    private_field: Type

    fn init(param: Type) {
        this.field = param
    }

    fn method(self) -> ReturnType {
        return this.field
    }

    fn static_method() -> ReturnType {
        return value
    }
}

// Inheritance
class DerivedClass extends BaseClass {
    // Override methods
    override fn method(self) -> ReturnType {
        return super.method() + " extended"
    }
}


## 5. Modules and Namespaces

### 5.1 Module Declaration
logos
module my_module {
    // module contents
    pub fn public_function() { }
    fn private_function() { }
}

// Import
import my_module::public_function
import my_module::{func1, func2}
import my_module::*  // Wildcard import (discouraged)


### 5.2 Visibility
- pub: Public visibility
- Private by default

## 6. Advanced Features

### 6.1 Traits (Interfaces)
logos
trait Display {
    fn display(self) -> String
}

impl Display for MyType {
    fn display(self) -> String {
        return "MyType instance"
    }
}


### 6.2 Pattern Matching
logos
// Tuple patterns
let (x, y) = (1, 2)

// Array patterns
let [first, second, ...rest] = [1, 2, 3, 4, 5]

// Record patterns
let Point{x, y} = Point{x: 1.0, y: 2.0}

// Nested patterns
match value {
    Some(Point{x, y}) if x > 0 => "Positive x coordinate",
    _ => "Other case"
}


### 6.3 Concurrency Primitives

#### 6.3.1 Actors
logos
actor Counter {
    count: Int = 0

    fn increment(self) {
        self.count = self.count + 1
    }

    fn get_count(self) -> Int {
        return self.count
    }
}

// Usage
let counter = Counter.spawn()
counter.increment()
let count = counter.get_count()


#### 6.3.2 Channels
logos
// Channel creation
let (sender, receiver) = channel<T>()

// Sending
await sender.send(value)

// Receiving
let value = await receiver.receive()


### 6.4 Multi-Language Integration
logos
// Call external languages
let python_result = @python("import math; math.sqrt(16)")
let rust_result = @rust("42 + 8")
let js_result = @js("Math.max(1, 2, 3)")
let go_result = @go("42 * 2")
let java_result = @java("java.lang.Math.abs(-42)")


### 6.5 Effects and Handlers
logos
effect FileOps {
    Read(path: String) -> String
    Write(path: String, content: String) -> Unit
}

fn read_file_safe(path: String) -> String {
    try {
        return perform FileOps.Read(path)
    } with FileOps {
        Read(path) -> {
            if file_exists(path) {
                resume(read_from_disk(path))
            } else {
                raise(FileNotFound(path))
            }
        }
    }
}


## 7. Standard Library

### 7.1 Core Modules
- std::collections: Arrays, maps, sets, etc.
- std::io: Input/output operations
- std::net: Network programming
- std::fs: File system operations
- std::thread: Threading and concurrency
- std::time: Time and date operations
- std::process: Process management
- std::env: Environment variables

### 7.2 Collection Operations
logos
// Array operations
let mapped = arr.map(|x| x * 2)
let filtered = arr.filter(|x| x > 0)
let reduced = arr.reduce(0, |acc, x| acc + x)
let zipped = arr1.zip(arr2)

// Functional combinators
let result = [1, 2, 3, 4, 5]
    |> filter(|x| x % 2 == 0)
    |> map(|x| x * x)
    |> reduce(0, |acc, x| acc + x)


## 8. Error Handling

### 8.1 Result Type
logos
fn divide(a: Float, b: Float) -> Result<Float, String> {
    if b == 0.0 {
        return Err("Division by zero")
    }
    return Ok(a / b)
}

// Usage
match divide(10.0, 2.0) {
    Ok(result) => print("Result: ${result}"),
    Err(error) => print("Error: ${error}")
}


### 8.2 Option Type
logos
fn find_item<T>(arr: [T], predicate: fn(T) -> Bool) -> Option<T> {
    for item in arr {
        if predicate(item) {
            return Some(item)
        }
    }
    return None
}

// Usage
match find_item([1, 2, 3, 4], |x| x > 2) {
    Some(item) => print("Found: ${item}"),
    None => print("Not found")
}


## 9. Memory Management

### 9.1 Ownership
- Each value has a single owner
- When owner goes out of scope, value is dropped
- Values can be moved but not copied by default

### 9.2 Borrowing
- References allow temporary access without taking ownership
- Multiple immutable references allowed
- Only one mutable reference allowed at a time

### 9.3 Lifetimes
- Compiler ensures references dont outlive the data they point to
- Explicit lifetime annotations when needed

## 10. Concurrency Model

### 10.1 Async/Await
logos
async fn fetch_data(url: String) -> Result<String, Error> {
    let response = await http_client.get(url)
    return response.text()
}

async fn main() {
    let result = await fetch_data("https://api.example.com")
    print(result.unwrap_or_else(|e| "Error: ${e}"))
}


### 10.2 Parallel Execution
logos
async fn process_multiple() {
    let [result1, result2, result3] = await Promise.all([
        process_task1(),
        process_task2(),
        process_task3()
    ])

    print("${result1}, ${result2}, ${result3}")
}


This specification provides a comprehensive overview of the Logos programming language, highlighting its advanced features that improve upon the original Elixir concept.
