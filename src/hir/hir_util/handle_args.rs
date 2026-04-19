use crate::ast::ArgType;
use crate::hir::hir_types::{FuncBlock, Id, Type, ValueId};

pub fn parse_type(type_str: &str) -> Option<Type> {
    match type_str {
        "Int" => Some(Type::Int),
        "Float" => Some(Type::Float),
        "String" => Some(Type::String),
        "DocElement" => Some(Type::DocElement),
        _ => None,
    }
}

pub fn handle_args(arguments: &Vec<ArgType>, ir_body: &mut FuncBlock) -> Vec<Id> {
    let mut args = Vec::new();
    for crate::ast::ArgType { name, ty } in arguments {
        // TODO handle cases where raw arguments are passed in
        // maybe look at instead of passing "arg" pass the variable type or
        // somethig if the var is not decalred, pass "var" if declared
        // for right now if there is a quotes or number, assume raw arg
        //
        // update: raw args are captured but in a shitty way, now just saying:
        // raw_arg_{index}
        match ty.as_str() {
            "var" => {
                for table in self.symbol_table.iter_mut().rev() {
                    if let Some(symbol) = table.get(name) {
                        match symbol {
                            Id::Value(id) => {
                                args.push(Id::Value(id));
                            }
                            _ => {}
                        }
                    }
                }
            }
            "int" => {
                let value = name.as_str().parse::<i64>().unwrap();
                let id = ValueId(ir_body.ops.len());
                let var_name = format!("raw_arg_{}", id.0);
                let var = self
                    .assign_local(
                        // TODO: this is overstretching
                        var_name.clone(),
                        crate::ast::Expression::new(
                            crate::ast::ExpressionKind::Int(value),
                            crate::diagnostic::SourceLocation::new(0, 0, self.ast.file.clone()),
                        ),
                        Id::Value(id),
                        false,
                    )
                    .unwrap();
                ir_body.ops.push(var);
                args.push(Id::Value(id));
            }
            "float" => {
                let value = name.as_str().parse::<f64>().unwrap();
                let id = ValueId(ir_body.ops.len());
                let var_name = format!("raw_arg_{}", id.0);
                let var = self
                    .assign_local(
                        // TODO: this is overstretching
                        var_name.clone(),
                        crate::ast::Expression::new(
                            crate::ast::ExpressionKind::Float(value),
                            crate::diagnostic::SourceLocation::new(0, 0, ast.file.clone()),
                        ),
                        Id::Value(id),
                        false,
                    )
                    .unwrap();
                ir_body.ops.push(var);
                args.push(Id::Value(id));
            }
            "string" => {
                let value = name
                    .as_str()
                    .parse::<String>()
                    .unwrap()
                    .trim_matches('"')
                    .to_string();
                let id = ValueId(ir_body.ops.len());
                let var_name = format!("raw_arg_{}", id.0);
                let var = self
                    .assign_local(
                        // TODO: this is overstretching
                        var_name.clone(),
                        crate::ast::Expression::new(
                            crate::ast::ExpressionKind::StringLiteral(value),
                            crate::diagnostic::SourceLocation::new(0, 0, self.ast.file.clone()),
                        ),
                        Id::Value(id),
                        false,
                    )
                    .unwrap();
                ir_body.ops.push(var);
                args.push(Id::Value(id));
            }
            _ => {}
        }
    }
    args
}
