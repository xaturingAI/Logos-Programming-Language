// Logos Programming Language Abstract Syntax Tree (AST)
// This module defines the data structures that represent the structure of Logos programs.
// Each AST node corresponds to a syntactic construct in the language.

use std::collections::HashMap;

/// Represents the entire program as a sequence of statements
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,  // The top-level statements in the program
}

/// Represents different kinds of statements in the language
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),  // An expression used as a statement (e.g., function call)
    
    /// Variable binding: let/mut name: Type = value
    LetBinding {
        mutable: bool,                    // Whether the variable is mutable (mut) or immutable (let)
        name: String,                     // The variable name
        type_annotation: Option<Type>,    // Optional type annotation
        value: Expression,                // The value to bind
        ownership_modifier: Option<OwnershipModifier>, // How the value is owned/borrowed
        lifetime_annotation: Option<String>, // Lifetime annotation if any
    },
    
    /// Constant binding: const name: Type = value
    ConstBinding { 
        name: String,                     // The constant name
        type_annotation: Option<Type>,    // Optional type annotation
        value: Expression,                // The value to bind
    },
    
    Function(FunctionDef),     // Function definition
    Class(ClassDef),         // Class definition
    Trait(TraitDef),         // Trait definition
    Implementation(ImplDef), // Implementation block
    Actor(ActorDef),         // Actor definition
    Effect(EffectDef),       // Effect definition
    Return(Option<Expression>), // Return statement with optional value
    Break,                   // Break statement
    Continue,                // Continue statement
    Block(Vec<Statement>),   // Block of statements { ... }
    MacroDefinition(MacroDef), // Macro definition
}

/// Represents different kinds of expressions in the language
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literal expressions - represent constant values
    Integer(i64),           // Integer literal (e.g., 42)
    Float(f64),             // Floating-point literal (e.g., 3.14)
    String(String),         // String literal (e.g., "hello")
    Boolean(bool),          // Boolean literal (true/false)
    Nil,                    // Nil/null value
    Char(char),             // Character literal (e.g., 'a')
    Array(Vec<Expression>), // Array literal (e.g., [1, 2, 3])
    Tuple(Vec<Expression>), // Tuple literal (e.g., (1, "hello", true))
    Struct(String, Vec<(String, Expression)>), // Struct instantiation (e.g., Point { x: 5, y: 10 })
    Lambda(Vec<Parameter>, Vec<Statement>), // Anonymous function (e.g., |x, y| { x + y })

    // Variable and operation expressions
    Identifier(String),     // Variable identifier (e.g., x, func_name)
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>), // Binary operation (e.g., a + b)
    UnaryOp(UnaryOp, Box<Expression>),                  // Unary operation (e.g., -x, !flag)
    Call(String, Vec<Expression>),                      // Function call (e.g., func(arg1, arg2))
    MethodCall(Box<Expression>, String, Vec<Expression>), // Method call (e.g., obj.method(args))
    FieldAccess(Box<Expression>, String),               // Field access (e.g., obj.field)


    // Actor messaging expressions for concurrent programming
    Spawn(String, Vec<Expression>),  // Spawn an actor with initial arguments
    Send(Box<Expression>, Box<Expression>),  // Send message to actor: send(actor, message)
    Receive,  // Receive a message (would be used in actor handlers)

    // Control flow expressions
    If(Box<Expression>, Vec<Statement>, Vec<Statement>), // condition, then, else
    Match(Box<Expression>, Vec<(Pattern, Option<Box<Expression>>, Vec<Statement>)>), // expr, (pattern, guard, body)
    BlockExpr(Vec<Statement>),                          // Expression block
    Block(Vec<Statement>),                              // Block expression

    // Multi-language integration expressions
    MultiLangCall(String, String),                      // @lang{code} - call code in another language
    MultiLangImport(String, String, Option<String>),    // @import("resource") - import from another language
    MultiLangIndex(String, String),                     // @index("resource") - index another language resource

    // Enhanced syntax constructs
    LambdaSimple(Vec<String>, Box<Expression>),         // Simple lambda: |args| expr
    Pipeline(Box<Expression>, Vec<Expression>),         // Pipeline operator: expr |> func1 |> func2
    BackPipeline(Box<Expression>, Vec<Expression>),     // Backward pipeline: func2 <| func1 <| expr
    DestructureAssignment(Box<Pattern>, Box<Expression>, Box<Statement>), // Destructuring: let (x, y) = expr in stmt
    InterpolatedString(Vec<StringPart>),                // Enhanced string interpolation: "text ${expr} more text"

    // CSP-style channel operations for concurrent programming
    ChannelCreate(Box<Type>),                           // Create a new channel: chan T
    ChannelSend(Box<Expression>, Box<Expression>),      // Send to channel: ch <- value
    ChannelReceive(Box<Expression>),                    // Receive from channel: <-ch
    ChannelClose(Box<Expression>),                      // Close a channel: close(ch)
    Select(Vec<SelectArm>),                             // Select statement for multiple channels

    // Async/Await constructs for asynchronous programming
    AsyncBlock(Vec<Statement>),                         // async { ... } block
    Await(Box<Expression>),                             // await expression
    Future(Box<Expression>),                            // future expression
    SpawnTask(Box<Expression>),                         // spawn a task (different from actor Spawn)
    Join(Box<Expression>),                              // join a spawned task
    Race(Vec<Expression>),                              // race multiple futures
    Timeout(Box<Expression>, Box<Expression>),          // timeout with duration

    // Metaprogramming and macro constructs
    MacroInvocation(String, Vec<Expression>),          // Macro call: macro_name!(args...)
}

/// Represents different types in the language
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,                    // 64-bit signed integer
    Float,                  // 64-bit floating-point number
    Bool,                   // Boolean type
    String,                 // String type
    Unit,                   // Unit type (equivalent to void)
    Array(Box<Type>),       // Array type: [T]
    Tuple(Vec<Type>),       // Tuple type: (T1, T2, ...)
    Function(Vec<Type>, Box<Type>), // Function type: (param1, param2, ...) -> return_type
    
    // Channel type for CSP-style concurrency
    Channel(Box<Type>),     // Channel type: chan T
    
    // Dependent types for advanced type systems
    Pi(Box<Parameter>, Box<Type>),        // Dependent function type: (x: A) -> B(x)
    Sigma(Box<Parameter>, Box<Type>),     // Dependent pair type: (x: A, B(x))
    Universe(u32),                        // Universe level: Type_0, Type_1, etc.
    Equality(Box<Type>, Box<Expression>, Box<Expression>), // Equality type: x =_A y
    
    // Linear types for resource management
    Linear(Box<Type>),                    // Linear type: !A (consumed exactly once)
    
    Named(String),                        // Named type (user-defined types)
    Generic(String),                      // Generic type parameter
    Option(Box<Type>),                    // Optional type: Option<T>
    Result(Box<Type>, Box<Type>),         // Result type: Result<T, E>
    Infer,                                // For type inference
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
            Type::String => write!(f, "String"),
            Type::Unit => write!(f, "Unit"),
            Type::Array(t) => write!(f, "[{}]", t),
            Type::Tuple(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", type_strs.join(", "))
            },
            Type::Function(params, ret) => {
                let param_strs: Vec<String> = params.iter().map(|t| t.to_string()).collect();
                write!(f, "({}) -> {}", param_strs.join(", "), ret)
            },
            Type::Channel(t) => write!(f, "chan<{}>", t),
            Type::Pi(param, ret) => write!(f, "({}: {}) -> {}", param.name, param.type_annotation, ret),
            Type::Sigma(param, snd) => write!(f, "({}: {}, {})", param.name, param.type_annotation, snd),
            Type::Universe(level) => write!(f, "Type{}", level),
            Type::Equality(ty, _, _) => write!(f, "Equal<{}>", ty), // Simplified representation
            Type::Linear(t) => write!(f, "!{}", t),
            Type::Named(name) => write!(f, "{}", name),
            Type::Generic(name) => write!(f, "{}", name),
            Type::Option(t) => write!(f, "Option<{}>", t),
            Type::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            Type::Infer => write!(f, "_"),
        }
    }
}

/// Represents parts of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),                       // Plain text part of the string
    Interpolated(Box<Expression>),         // Expression part of the string: ${expr}
}

/// Represents binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,              // Arithmetic operators: +, -, *, /, %
    Eq, Ne, Lt, Gt, Le, Ge, Spaceship,   // Comparison operators: ==, !=, <, >, <=, >=, <=>
    And, Or,                             // Logical operators: &&, ||
    PipeForward, PipeBackward,           // Pipeline operators: |>, <|
    Power,                               // Exponentiation: ^
    Range,                               // Range operator: ..
}

/// Represents unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,                                  // Negation: -
    Not,                                  // Logical NOT: !
    Ref,                                  // Reference: &
    Deref,                                // Dereference: *
}

/// Represents a function definition
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,                     // Function name
    pub parameters: Vec<Parameter>,       // Function parameters
    pub return_type: Option<Type>,        // Optional return type annotation
    pub body: Vec<Statement>,             // Function body statements
    pub is_async: bool,                   // Whether the function is async
    pub is_public: bool,                  // Whether the function is public
    pub is_awaitable: bool,               // For functions that can be awaited
    pub effect_annotations: Vec<EffectAnnotation>, // Annotations for structured concurrency
}

/// Represents different effect annotations for functions
#[derive(Debug, Clone, PartialEq)]
pub enum EffectAnnotation {
    Async,                                 // Async function
    Sync,                                  // Synchronous function
    ThreadSafe,                           // Thread-safe function
    SideEffectFree,                       // Function with no side effects
    IOBound,                              // I/O bound function
    CPUBound,                             // CPU bound function
    Blocking,                             // Blocking function
    NonBlocking,                          // Non-blocking function
}

/// Represents an async block
#[derive(Debug, Clone)]
pub struct AsyncBlock {
    pub body: Vec<Statement>,             // Statements in the async block
    pub effect_context: EffectContext,    // Context for effect handling
}

/// Represents different effect contexts
#[derive(Debug, Clone)]
pub enum EffectContext {
    Sequential,                           // Sequential execution
    Concurrent,                           // Concurrent execution
    Parallel,                             // Parallel execution
    Distributed,                          // Distributed execution
}

/// Represents an await expression
#[derive(Debug, Clone)]
pub struct AwaitExpression {
    pub expression: Box<Expression>,      // The expression to await
}

/// Represents a future type
#[derive(Debug, Clone)]
pub struct FutureType {
    pub inner_type: Box<Type>,            // The type of the future's result
}

/// Represents a channel type
#[derive(Debug, Clone)]
pub struct ChannelType {
    pub element_type: Box<Type>,          // The type of elements in the channel
}

/// Represents a select statement for channels
#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub arms: Vec<SelectArm>,             // Arms of the select statement
    pub default_case: Option<Vec<Statement>>, // Optional default case
}

/// Represents an arm of a select statement
#[derive(Debug, Clone, PartialEq)]
pub struct SelectArm {
    pub channel_operation: ChannelOperation, // Operation on a channel
    pub pattern: Option<Pattern>,            // Optional pattern to match
    pub body: Vec<Statement>,                // Body to execute when selected
}

/// Represents different channel operations
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelOperation {
    Send { channel: Box<Expression>, value: Box<Expression> },  // Send operation
    Receive { channel: Box<Expression> },                       // Receive operation
    Close { channel: Box<Expression> },                         // Close operation
}

/// Represents a function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,                     // Parameter name
    pub type_annotation: Type,            // Type of the parameter
    pub ownership_modifier: Option<OwnershipModifier>, // How the parameter is owned/borrowed
    pub lifetime_annotation: Option<String>, // Lifetime annotation
    pub default_value: Option<Expression>, // Optional default value
    pub mutability: Option<Mutability>,   // Explicit mutability annotation for parameters
}

/// Represents different mutability options for parameters and variables
#[derive(Debug, Clone, PartialEq)]
pub enum Mutability {
    Immutable,    // Immutable (default)
    Mutable,      // Mutable (mut keyword)
    RefMutable,   // Mutable reference (&mut)
    Linear,       // Linear type (consumed exactly once)
}

/// Represents different ownership modifiers for parameters
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipModifier {
    Owned,                                // Owns the value (takes ownership)
    Borrowed,                            // Borrows the value immutably
    MutablyBorrowed,                     // Borrows the value mutably
    Shared,                              // Shares the value (reference counted)
    Linear,                              // Linear ownership (used exactly once)
    Moved,                               // Moved ownership (value has been moved)
    Transferred,                         // Transferred ownership to another entity
}

/// Represents a class definition
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef {
    pub name: String,                     // Class name
    pub fields: Vec<FieldDef>,            // Class fields
    pub methods: Vec<FunctionDef>,        // Class methods
    pub parent: Option<String>,           // Optional parent class for inheritance
    pub access_modifier: AccessModifier,  // Access level (public, private, etc.)
    pub is_abstract: bool,                // Whether the class is abstract
    pub generics: Vec<String>,            // Generic type parameters
    pub interfaces: Vec<String>,          // Interfaces this class implements
    pub constructors: Vec<ConstructorDef>, // Constructor methods
    pub destructors: Vec<DestructorDef>,   // Destructor methods
}

/// Represents a field definition in a class
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub name: String,                     // Field name
    pub type_annotation: Type,            // Field type
    pub access_modifier: AccessModifier,  // Access level (public, private, etc.)
    pub is_mutable: bool,                 // Whether the field is mutable
    pub is_static: bool,                  // Whether the field is static
    pub default_value: Option<Expression>, // Optional default value
}

/// Represents a constructor definition
#[derive(Debug, Clone, PartialEq)]
pub struct ConstructorDef {
    pub parameters: Vec<Parameter>,       // Constructor parameters
    pub body: Vec<Statement>,             // Constructor body
    pub access_modifier: AccessModifier,  // Access level
}

/// Represents a destructor definition
#[derive(Debug, Clone, PartialEq)]
pub struct DestructorDef {
    pub body: Vec<Statement>,             // Destructor body
    pub access_modifier: AccessModifier,  // Access level
}

/// Represents access modifiers for class members
#[derive(Debug, Clone, PartialEq)]
pub enum AccessModifier {
    Public,                               // Public access
    Private,                              // Private access
    Protected,                            // Protected access
    Internal,                             // Internal access
}

/// Represents a trait definition
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: String,                     // Trait name
    pub type_params: Vec<String>,         // Generic type parameters
    pub methods: Vec<FunctionDef>,        // Method signatures in the trait
    pub associated_types: Vec<AssociatedTypeDef>, // Associated types
    pub super_traits: Vec<String>,        // Super traits (inheritance)
}

/// Associated type definition within a trait
#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedTypeDef {
    pub name: String,
    pub bounds: Vec<String>,  // Trait bounds
    pub default: Option<Type>, // Default type if any
}

/// Represents an implementation block
#[derive(Debug, Clone, PartialEq)]
pub struct ImplDef {
    pub trait_name: String,               // Name of the trait being implemented
    pub for_type: String,                 // Type that implements the trait
    pub type_params: Vec<String>,         // Generic type parameters
    pub methods: Vec<FunctionDef>,        // Implemented methods
    pub associated_types: Vec<(String, Type)>, // Implemented associated types
}

/// Represents an actor definition
#[derive(Debug, Clone, PartialEq)]
pub struct ActorDef {
    pub name: String,                     // Actor name
    pub state: Vec<(String, Type)>,       // Actor state fields (name, type)
    pub handlers: Vec<FunctionDef>,       // Message handlers
}

/// Represents an effect definition
#[derive(Debug, Clone, PartialEq)]
pub struct EffectDef {
    pub name: String,                     // Effect name
    pub operations: Vec<FunctionDef>,     // Effect operations
}

/// Represents a macro definition
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub name: String,                     // Macro name
    pub parameters: Vec<String>,          // Macro parameters (for pattern matching)
    pub body: Vec<Statement>,             // Macro body (template for expansion)
    pub is_hygienic: bool,               // Whether the macro follows hygienic rules
}

/// Represents different pattern types for pattern matching
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Identifier(String),                    // Variable pattern: x
    Literal(Expression),                   // Literal pattern: 42, "hello", true
    Wildcard,                            // Wildcard pattern: _
    Tuple(Vec<Pattern>),                 // Tuple pattern: (x, y, z)
    Array(Vec<Pattern>),                 // Array pattern: [x, y, z]
    Struct(String, Vec<(String, Pattern)>), // Struct pattern: Point { x, y }
    Or(Box<Pattern>, Box<Pattern>),      // Or pattern: pattern1 | pattern2
}


// Helper functions for creating AST nodes
impl Program {
    /// Creates a new program with the given statements
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl Statement {
    /// Creates an expression statement
    pub fn expr(expr: Expression) -> Self {
        Statement::Expression(expr)
    }

    /// Creates a return statement with an optional expression
    pub fn return_expr(expr: Option<Expression>) -> Self {
        Statement::Return(expr)
    }
}

impl Expression {
    /// Creates an integer literal expression
    pub fn int(value: i64) -> Self {
        Expression::Integer(value)
    }

    /// Creates a float literal expression
    pub fn float(value: f64) -> Self {
        Expression::Float(value)
    }

    /// Creates a string literal expression
    pub fn string(value: String) -> Self {
        Expression::String(value)
    }

    /// Creates a boolean literal expression
    pub fn bool(value: bool) -> Self {
        Expression::Boolean(value)
    }

    /// Creates an identifier expression
    pub fn ident(name: String) -> Self {
        Expression::Identifier(name)
    }

    /// Creates a binary operation expression
    pub fn binary(left: Expression, op: BinaryOp, right: Expression) -> Self {
        Expression::BinaryOp(Box::new(left), op, Box::new(right))
    }

    /// Creates a function call expression
    pub fn call(name: String, args: Vec<Expression>) -> Self {
        Expression::Call(name, args)
    }
}

impl Type {
    /// Creates an integer type
    pub fn int() -> Self {
        Type::Int
    }

    /// Creates a float type
    pub fn float() -> Self {
        Type::Float
    }

    /// Creates a boolean type
    pub fn bool() -> Self {
        Type::Bool
    }

    /// Creates a string type
    pub fn string() -> Self {
        Type::String
    }

    /// Creates a unit type
    pub fn unit() -> Self {
        Type::Unit
    }

    /// Creates an array type
    pub fn array(inner: Type) -> Self {
        Type::Array(Box::new(inner))
    }

    /// Creates a named type
    pub fn named(name: String) -> Self {
        Type::Named(name)
    }
}

impl BinaryOp {
    /// Creates an addition operator
    pub fn add() -> Self {
        BinaryOp::Add
    }

    /// Creates a subtraction operator
    pub fn sub() -> Self {
        BinaryOp::Sub
    }

    /// Creates a multiplication operator
    pub fn mul() -> Self {
        BinaryOp::Mul
    }

    /// Creates an equality operator
    pub fn eq() -> Self {
        BinaryOp::Eq
    }

    /// Creates a logical AND operator
    pub fn and() -> Self {
        BinaryOp::And
    }

    /// Creates a logical OR operator
    pub fn or() -> Self {
        BinaryOp::Or
    }
}

impl UnaryOp {
    /// Creates a negation operator
    pub fn neg() -> Self {
        UnaryOp::Neg
    }

    /// Creates a logical NOT operator
    pub fn not() -> Self {
        UnaryOp::Not
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_creation() {
        let program = Program::new(vec![
            Statement::LetBinding {
                mutable: false,
                name: "x".to_string(),
                type_annotation: Some(Type::int()),
                value: Expression::int(42),
            },
            Statement::expr(Expression::call(
                "print".to_string(),
                vec![Expression::string("Hello, World!".to_string())]
            )),
        ]);

        assert_eq!(program.statements.len(), 2);

        match &program.statements[0] {
            Statement::LetBinding { value, .. } => {
                match value {
                    Expression::Integer(42) => {},
                    _ => panic!("Expected Integer(42)"),
                }
            },
            _ => panic!("Expected LetBinding"),
        }
    }
}
// Additional definitions for missing expression types
