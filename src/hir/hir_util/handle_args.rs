use crate::ast::{ArgType, Type as AstType};
use crate::diagnostic::SemanticError;
use crate::hir::hir_types::{FuncBlock, Literal, Op, Type, ValueId};

pub fn parse_type(ty: &AstType) -> Option<Type> {
    match ty {
        AstType::Int => Some(Type::Int),
        AstType::Float => Some(Type::Float),
        AstType::String => Some(Type::String),
        AstType::DocElem => Some(Type::DocElement),
        AstType::Var => None,
    }
}

pub fn handle_args(
    arguments: &[ArgType],
    ir_body: &mut FuncBlock,
) -> Result<Vec<ValueId>, Vec<SemanticError>> {
    let mut args = Vec::new();
    let mut errors = Vec::new();

    for arg in arguments {
        match arg.ty {
            crate::ast::Type::Var => {
                //  TODO Identifier argument resolution belongs in validation/name resolution.
                let id = ValueId(ir_body.ops.len());
                ir_body.ops.push(Op::VarRef {
                    id,
                    name: arg.name.clone(),
                });
                args.push(id);
            }
            crate::ast::Type::Int => {
                if let Ok(value) = arg.name.parse::<i64>() {
                    let id = ValueId(ir_body.ops.len());
                    ir_body.ops.push(Op::Const {
                        id,
                        name: format!("raw_arg_{}", ir_body.ops.len()),
                        literal: Literal::Int(value),
                        ty: Type::Int,
                    });
                    args.push(id);
                }
            }
            crate::ast::Type::Float => {
                if let Ok(value) = arg.name.parse::<f64>() {
                    let id = ValueId(ir_body.ops.len());
                    ir_body.ops.push(Op::Const {
                        id,
                        name: format!("raw_arg_{}", ir_body.ops.len()),
                        literal: Literal::Float(value),
                        ty: Type::Float,
                    });
                    args.push(id);
                }
            }
            crate::ast::Type::String => {
                let id = ValueId(ir_body.ops.len());
                ir_body.ops.push(Op::Const {
                    id,
                    name: format!("raw_arg_{}", ir_body.ops.len()),
                    literal: Literal::String(arg.name.clone()),
                    ty: Type::String,
                });
                args.push(id);
            }
            crate::ast::Type::DocElem => {}
        }
    }

    if errors.is_empty() {
        Ok(args)
    } else {
        Err(errors)
    }
}
