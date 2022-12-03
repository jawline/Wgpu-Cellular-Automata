#[derive(Debug, Clone)]
pub enum Expr {
    U32(u32),
    Alive,
    Neighbors,
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
}

use Expr::*;

impl Expr {
    pub fn to_shader(&self) -> String {
        match self {
            U32(val) => format!("{}u", val),
            Alive => format!("is_alive"),
            Neighbors => format!("num_neighbors"),
            Gt(lhs, rhs) => format!(
                "u32(({}) > ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
            Gte(lhs, rhs) => format!(
                "u32(({}) >= ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
            Lt(lhs, rhs) => format!(
                "u32(({}) < ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
            Lte(lhs, rhs) => format!(
                "u32(({}) <= ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
            And(lhs, rhs) => format!("(({}) & ({}))", Self::to_shader(lhs), Self::to_shader(rhs)),
            Or(lhs, rhs) => format!("(({}) | ({}))", Self::to_shader(lhs), Self::to_shader(rhs)),
            Equal(lhs, rhs) => format!(
                "u32(({}) == ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
        }
    }
}

pub fn const_u32(value: u32) -> Expr {
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

pub fn and(lhs: Expr, rhs: Expr) -> Expr {
    And(Box::new(lhs), Box::new(rhs))
}

pub fn or(lhs: Expr, rhs: Expr) -> Expr {
    Or(Box::new(lhs), Box::new(rhs))
}

pub fn equal(lhs: Expr, rhs: Expr) -> Expr {
    Equal(Box::new(lhs), Box::new(rhs))
}

#[derive(Debug, Clone)]
pub enum Statement {
    Void,
    SetResult(Expr),
    IfThenElse {
        condition: Expr,
        if_true_then: Box<Statement>,
        if_false_then: Box<Statement>,
    },
}

use Statement::*;

impl Statement {
    pub fn to_shader(&self) -> String {
        match self {
            Void => format!(""),
            SetResult(expr) => format!("result = {};", expr.to_shader()),
            IfThenElse {
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

pub fn void() -> Statement {
    Void
}

pub fn set_result(expr: Expr) -> Statement {
    SetResult(expr)
}

pub fn if_then_else(
    condition: Expr,
    if_true_then: Statement,
    if_false_then: Statement,
) -> Statement {
    IfThenElse {
        condition,
        if_true_then: Box::new(if_true_then),
        if_false_then: Box::new(if_false_then),
    }
}
