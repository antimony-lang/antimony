#[derive(Debug)]
pub struct Program {
    pub func: Vec<Function>,
    pub globals: Vec<String>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<Variable>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Statement {
    Declare(Variable, Option<Expression>),
    Return(Expression),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
    Exp(Expression),
    Compound(Vec<Statement>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expression {
    Int(u32),
    Str(String),
    Char(u8),
    FunctionCall(String, Vec<Expression>),
    Variable(String),
    VariableRef(String),
    Assign(String, Box<Expression>),
    AssignPostfix(String, Box<Expression>),
    Ternary(Box<Expression>, Box<Expression>, Box<Expression>),
}
