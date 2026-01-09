use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    LetBinding { mutable: bool, name: String, type_annotation: Option<Type>, value: Expression },
    ConstBinding { name: String, type_annotation: Option<Type>, value: Expression },
    Function(FunctionDef),
    Class(ClassDef),
    Trait(TraitDef),
    Implementation(ImplDef),
    Actor(ActorDef),
    Effect(EffectDef),
    Return(Option<Expression>),
    Break,
    Continue,
    Block(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Nil,

    // Variables and operations
    Identifier(String),
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>),
    UnaryOp(UnaryOp, Box<Expression>),
    Call(String, Vec<Expression>),
    MethodCall(Box<Expression>, String, Vec<Expression>),
    FieldAccess(Box<Expression>, String),

    // Tuples
    Tuple(Vec<Expression>),

    // Actor messaging
    Spawn(String, Vec<Expression>),  // Spawn an actor with initial arguments
    Send(Box<Expression>, Box<Expression>),  // Send message to actor: send(actor, message)
    Receive,  // Receive a message (would be used in actor handlers)

    // Control flow
    If(Box<Expression>, Vec<Statement>, Vec<Statement>), // condition, then, else
    Match(Box<Expression>, Vec<(Pattern, Option<Box<Expression>>, Vec<Statement>)>), // expr, (pattern, guard, body)
    Lambda(Vec<Parameter>, Vec<Statement>),
    BlockExpr(Vec<Statement>),

    // Multi-language integration
    MultiLangCall(String, String), // language, code
    MultiLangImport(String, String, Option<String>), // language, package/module, alias
    MultiLangIndex(String, String), // language, indexed_resource (for codex/indexing)

    // Enhanced syntax constructs
    LambdaSimple(Vec<String>, Box<Expression>),  // Simple lambda: |args| expr
    Pipeline(Box<Expression>, Vec<Expression>),  // Pipeline operator: expr |> func1 |> func2
    BackPipeline(Box<Expression>, Vec<Expression>),  // Backward pipeline: func2 <| func1 <| expr
    DestructureAssignment(Box<Pattern>, Box<Expression>, Box<Statement>),  // Destructuring: let (x, y) = expr in stmt
    InterpolatedString(Vec<StringPart>),  // Enhanced string interpolation: "text ${expr} more text"

    // CSP-style channel operations
    ChannelCreate(Box<Type>),              // Create a new channel: chan T
    ChannelSend(Box<Expression>, Box<Expression>),  // Send to channel: ch <- value
    ChannelReceive(Box<Expression>),       // Receive from channel: <-ch
    ChannelClose(Box<Expression>),         // Close a channel: close(ch)
    Select(Vec<SelectArm>),               // Select statement for multiple channels

    // Async/Await constructs
    AsyncBlock(Vec<Statement>),            // async { ... } block
    Await(Box<Expression>),                // await expression
    Future(Box<Expression>),               // future expression
    SpawnTask(Box<Expression>),            // spawn a task (different from actor Spawn)
    Join(Box<Expression>),                 // join a spawned task
    Race(Vec<Expression>),                 // race multiple futures
    Timeout(Box<Expression>, Box<Expression>), // timeout with duration
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Unit,
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>), // params, return
    // Channel type for CSP-style concurrency
    Channel(Box<Type>),           // Channel type: chan T
    // Dependent types
    Pi(Box<Parameter>, Box<Type>),        // Dependent function type: (x: A) -> B(x)
    Sigma(Box<Parameter>, Box<Type>),     // Dependent pair type: (x: A, B(x))
    Universe(u32),                        // Universe level: Type_0, Type_1, etc.
    Equality(Box<Type>, Box<Expression>, Box<Expression>), // Equality type: x =_A y
    // Linear types
    Linear(Box<Type>),                    // Linear type: !A (consumed exactly once)
    Named(String),
    Generic(String),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Infer, // For type inference
}

#[derive(Debug, Clone)]
pub enum StringPart {
    Literal(String),
    Interpolated(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge, Spaceship,
    And, Or,
    PipeForward, PipeBackward,
    Power,
    Range,  // For .. operator
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg, Not, Ref, Deref,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
    pub is_async: bool,
    pub is_public: bool,
    pub is_awaitable: bool,  // For functions that can be awaited
    pub effect_annotations: Vec<EffectAnnotation>, // For structured concurrency
}

#[derive(Debug, Clone)]
pub enum EffectAnnotation {
    Async,
    Sync,
    ThreadSafe,
    SideEffectFree,
    IOBound,
    CPUBound,
    Blocking,
    NonBlocking,
}

#[derive(Debug, Clone)]
pub struct AsyncBlock {
    pub body: Vec<Statement>,
    pub effect_context: EffectContext,
}

#[derive(Debug, Clone)]
pub enum EffectContext {
    Sequential,
    Concurrent,
    Parallel,
    Distributed,
}

#[derive(Debug, Clone)]
pub struct AwaitExpression {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct FutureType {
    pub inner_type: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct ChannelType {
    pub element_type: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub arms: Vec<SelectArm>,
    pub default_case: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct SelectArm {
    pub channel_operation: ChannelOperation,
    pub pattern: Option<Pattern>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum ChannelOperation {
    Send { channel: Box<Expression>, value: Box<Expression> },
    Receive { channel: Box<Expression> },
    Close { channel: Box<Expression> },
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Type,
    pub ownership_modifier: Option<OwnershipModifier>,
    pub lifetime_annotation: Option<String>,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum OwnershipModifier {
    Owned,      // Owns the value (takes ownership)
    Borrowed,   // Borrows the value immutably
    MutablyBorrowed, // Borrows the value mutably
    Shared,     // Shares the value (reference counted)
    Linear,     // Linear ownership (used exactly once)
    Moved,      // Moved ownership (value has been moved)
}

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub methods: Vec<FunctionDef>,
    pub parent: Option<String>,
    pub access_modifier: AccessModifier,
    pub is_abstract: bool,
    pub generics: Vec<String>,  // For generic classes
    pub interfaces: Vec<String>, // For interface implementation
    pub constructors: Vec<ConstructorDef>, // Constructor methods
    pub destructors: Vec<DestructorDef>,   // Destructor methods
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_annotation: Type,
    pub access_modifier: AccessModifier,
    pub is_mutable: bool,
    pub is_static: bool,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct ConstructorDef {
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
    pub access_modifier: AccessModifier,
}

#[derive(Debug, Clone)]
pub struct DestructorDef {
    pub body: Vec<Statement>,
    pub access_modifier: AccessModifier,
}

#[derive(Debug, Clone)]
pub enum AccessModifier {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: String,
    pub methods: Vec<FunctionDef>,
}

#[derive(Debug, Clone)]
pub struct ImplDef {
    pub trait_name: String,
    pub for_type: String,
    pub methods: Vec<FunctionDef>,
}

#[derive(Debug, Clone)]
pub struct ActorDef {
    pub name: String,
    pub state: Vec<(String, Type)>,
    pub handlers: Vec<FunctionDef>,
}

#[derive(Debug, Clone)]
pub struct EffectDef {
    pub name: String,
    pub operations: Vec<FunctionDef>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(String),
    Literal(Expression),
    Wildcard,
    Tuple(Vec<Pattern>),
    Array(Vec<Pattern>),
    Struct(String, Vec<(String, Pattern)>),
    Or(Box<Pattern>, Box<Pattern>),
}


// Helper functions for creating AST nodes
impl Program {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl Statement {
    pub fn expr(expr: Expression) -> Self {
        Statement::Expression(expr)
    }
    
    pub fn return_expr(expr: Option<Expression>) -> Self {
        Statement::Return(expr)
    }
}

impl Expression {
    pub fn int(value: i64) -> Self {
        Expression::Integer(value)
    }
    
    pub fn float(value: f64) -> Self {
        Expression::Float(value)
    }
    
    pub fn string(value: String) -> Self {
        Expression::String(value)
    }
    
    pub fn bool(value: bool) -> Self {
        Expression::Boolean(value)
    }
    
    pub fn ident(name: String) -> Self {
        Expression::Identifier(name)
    }
    
    pub fn binary(left: Expression, op: BinaryOp, right: Expression) -> Self {
        Expression::BinaryOp(Box::new(left), op, Box::new(right))
    }
    
    pub fn call(name: String, args: Vec<Expression>) -> Self {
        Expression::Call(name, args)
    }
}

impl Type {
    pub fn int() -> Self {
        Type::Int
    }
    
    pub fn float() -> Self {
        Type::Float
    }
    
    pub fn bool() -> Self {
        Type::Bool
    }
    
    pub fn string() -> Self {
        Type::String
    }
    
    pub fn unit() -> Self {
        Type::Unit
    }
    
    pub fn array(inner: Type) -> Self {
        Type::Array(Box::new(inner))
    }
    
    pub fn named(name: String) -> Self {
        Type::Named(name)
    }
}

impl BinaryOp {
    pub fn add() -> Self {
        BinaryOp::Add
    }
    
    pub fn sub() -> Self {
        BinaryOp::Sub
    }
    
    pub fn mul() -> Self {
        BinaryOp::Mul
    }
    
    pub fn eq() -> Self {
        BinaryOp::Eq
    }
    
    pub fn and() -> Self {
        BinaryOp::And
    }
    
    pub fn or() -> Self {
        BinaryOp::Or
    }
}

impl UnaryOp {
    pub fn neg() -> Self {
        UnaryOp::Neg
    }
    
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