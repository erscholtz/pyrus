use crate::ast::Type as AstType;
use crate::hir::hir_types::Type;

pub fn parse_type(ty: &AstType) -> Option<Type> {
    match ty {
        AstType::Int => Some(Type::Int),
        AstType::Float => Some(Type::Float),
        AstType::String => Some(Type::String),
        AstType::DocElem => Some(Type::DocElement),
        AstType::Var => None,
    }
}
