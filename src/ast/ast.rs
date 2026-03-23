use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    Mod,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub enum Expression {
    StringLiteral(String),
    InterpolatedString(Vec<InterpPart>),
    Int(i64),
    Float(f64),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOp,
        expression: Box<Expression>,
    },
    StructDefault(String),
}

impl Expression {
    // TODO this is not good find a way to remove
    pub fn as_number(&self) -> Option<i64> {
        match self {
            Expression::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Expression::StringLiteral(s) => s.clone(),
            Expression::InterpolatedString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        InterpPart::Text(text) => result.push_str(text),
                        InterpPart::Expression(expr) => result.push_str(&expr.to_string()),
                    }
                }
                result
            }
            Expression::Unary {
                operator,
                expression,
            } => match operator {
                UnaryOp::Negate => format!("-{}", expression.to_string()),
                UnaryOp::Not => format!("!{}", expression.to_string()),
            },
            Expression::StructDefault(name) => format!("default({})", name),
            Expression::Int(value) => format!("{}", value),
            Expression::Float(value) => format!("{}", value),
            _ => "Error".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InterpPart {
    Text(String),
    Expression(Expression), // interpolated portion
}

#[derive(Debug, Clone)]
pub struct FuncParam {
    pub ty: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct TemplateBlock {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ArgType {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub enum Statement {
    /// everything between `{` and `}` that isn't a function definition or a return
    DefaultSet {
        key: String,
        value: Expression,
    },
    VarAssign {
        // value should never be an expression, should always be explicit
        name: String,
        value: Expression,
    },
    ConstAssign {
        name: String,
        value: Expression,
    },
    If {
        condition: Expression,
        body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        iterator: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Return {
        doc_element: DocElement, // TODO change this to be on expression
    },
    /// name(args) { body... }
    FunctionDecl {
        name: String,
        args: Vec<FuncParam>,
        body: Vec<Statement>,
        return_type: Option<String>,
    },
    ElementDecl {
        name: String,
        args: Vec<FuncParam>,
        body: Vec<Statement>,
    },
}

#[derive(Debug, Clone)]
pub enum DocElement {
    Text {
        content: String,
        attributes: HashMap<String, Expression>,
    },
    Image {
        src: String,
        attributes: HashMap<String, Expression>,
    },
    Table {
        rows: Vec<Vec<DocElement>>,
        attributes: HashMap<String, Expression>,
    },
    List {
        items: Vec<DocElement>,
        attributes: HashMap<String, Expression>,
    },
    Code {
        content: String,
        attributes: HashMap<String, Expression>,
    },
    Call {
        name: String,
        args: Vec<ArgType>,
    },
    Link {
        href: String,
        content: String,
        attributes: HashMap<String, Expression>,
    },
    Section {
        elements: Vec<DocElement>,
        attributes: HashMap<String, Expression>,
    },
}

// Document Block

#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector_list: Vec<Selector>,
    pub declaration_block: Vec<KeyValue>,
    pub specificity: usize, // Pre-computed specificity for cascade ordering
}

impl StyleRule {
    pub fn new(selector_list: Vec<Selector>, declaration_block: Vec<KeyValue>) -> Self {
        let specificity = Self::compute_specificity(&selector_list);
        Self {
            selector_list,
            declaration_block,
            specificity,
        }
    }

    /// Compute CSS specificity: (id_count, class_count, type_count) as a single number
    /// Returns: id_count * 100 + class_count * 10 + type_count
    fn compute_specificity(selectors: &[Selector]) -> usize {
        let mut id_count = 0;
        let mut class_count = 0;
        let mut type_count = 0;

        for selector in selectors {
            match selector {
                Selector::Id(_) => id_count += 1,
                Selector::Class(_) => class_count += 1,
                Selector::Type(_) => type_count += 1,
            }
        }

        id_count * 100 + class_count * 10 + type_count
    }
}

#[derive(Debug, Clone)]
pub enum Selector {
    Class(String),
    Id(String),
    Type(String),
}

#[derive(Debug, Clone)]
pub struct DocumentBlock {
    pub elements: Vec<DocElement>,
}

#[derive(Debug, Clone)]
pub struct StyleBlock {
    pub statements: Vec<StyleRule>,
}

#[derive(Debug, Clone)]
pub struct Ast {
    pub template: Option<TemplateBlock>,
    pub document: Option<DocumentBlock>,
    pub style: Option<StyleBlock>,
}
