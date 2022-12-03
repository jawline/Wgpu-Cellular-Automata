#[derive(Debug)]
pub enum Expr {
    U32(u32),
    Alive,
    Neighbors,
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
}

use Expr::*;

impl Expr {
    pub fn to_shader(&self) -> String {
        match self {
            U32(val) => format!("{}", val),
            Alive => format!("is_alive"),
            Neighbors => format!("num_neighbors"),
            Gt(lhs, rhs) => format!("{} > {}", Self::to_shader(lhs), Self::to_shader(rhs)),
            Gte(lhs, rhs) => format!("{} >= {}", Self::to_shader(lhs), Self::to_shader(rhs)),
            Lt(lhs, rhs) => format!("{} < {}", Self::to_shader(lhs), Self::to_shader(rhs)),
            Lte(lhs, rhs) => format!("{} <= {}", Self::to_shader(lhs), Self::to_shader(rhs)),
            Equal(lhs, rhs) => format!("{} == {}", Self::to_shader(lhs), Self::to_shader(rhs)),
        }
    }
}

pub fn u32(value: u32) -> Expr {
    U32(value)
}

pub fn alive() -> Expr {
    Alive
}

pub fn neighbors() -> Expr {
    Neighbors
}

pub fn gt(lhs: Expr, rhs: Expr) -> Expr {
    Gt(Box::new(lhs), Box::new(rhs))
}

pub fn gte(lhs: Expr, rhs: Expr) -> Expr {
    Gte(Box::new(lhs), Box::new(rhs))
}

pub fn lt(lhs: Expr, rhs: Expr) -> Expr {
    Lt(Box::new(lhs), Box::new(rhs))
}

pub fn lte(lhs: Expr, rhs: Expr) -> Expr {
    Lte(Box::new(lhs), Box::new(rhs))
}

pub fn equal(lhs: Expr, rhs: Expr) -> Expr {
    Equal(Box::new(lhs), Box::new(rhs))
}

#[derive(Debug)]
pub enum Statement {
    SetResult(Expr),
    If {
        condition: Expr,
        if_true_then: Box<Statement>,
        if_false_then: Box<Statement>,
    },
}

use Statement::*;

impl Statement {
    pub fn to_shader(&self) -> String {
        match self {
            SetResult(expr) => format!("result = {}", expr.to_shader()),
            If {
                condition,
                if_true_then,
                if_false_then,
            } => format!(
                "if ({}) {{ {} }} else {{ {} }}",
                condition.to_shader(),
                if_true_then.to_shader(),
                if_false_then.to_shader()
            ),
        }
    }
}
