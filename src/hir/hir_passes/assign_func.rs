use std::collections::HashMap;

use crate::hir::HIRPass_old;
use crate::hir::ir_types::{FuncBlock, HIRModule, Id, Op, ValueId};

use crate::ast::{ArgType, Expression, StatementKind};

impl<'ast_lifetime> HIRPass_old<'ast_lifetime> {
    pub fn lower_function_block(
        &mut self,
        body: &Vec<crate::ast::Statement>,
        hirmodule: &mut HIRModule,
    ) -> FuncBlock {
        let mut ir_body = FuncBlock {
            ops: Vec::new(),
            returned_element_ref: None,
        };

        self.symbol_table.push(HashMap::new()); // add new scope (function)

        for stmt in body {
            match &stmt.node {
                StatementKind::ConstAssign { name, value } => {
                    let id = Id::Value(ValueId(ir_body.ops.len()));
                    let value = self.assign_local(name.clone(), value.clone(), id, false);
                    ir_body.ops.push(value);
                    self.add_symbol(name.clone(), id);
                }
                StatementKind::VarAssign { name, value } => {
                    let id = Id::Value(ValueId(ir_body.ops.len()));
                    let value = self.assign_local(name.clone(), value.clone(), id, true);
                    ir_body.ops.push(value);
                    self.add_symbol(name.clone(), id);
                }

                StatementKind::Return { doc_element } => {
                    let element_id =
                        self.lower_document_element(doc_element, hirmodule, &mut ir_body, None);
                    ir_body.ops.push(Op::Return {
                        doc_element_ref: element_id,
                    });
                    ir_body.returned_element_ref = Some(element_id);
                }
                _ => {
                    todo!("other types not handled yet")
                }
            }
        }

        self.symbol_table.pop(); // remove scope (function)
        ir_body
    }

    pub fn handle_args(&mut self, arguments: &Vec<ArgType>, ir_body: &mut FuncBlock) -> Vec<Id> {
        self.symbol_table.push(HashMap::new()); // adding new table for arg scope
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
                                    args.push(Id::Value(*id));
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
                    let var = self.assign_local(
                        var_name.clone(),
                        crate::ast::Expression::new(
                            crate::ast::ExpressionKind::Int(value),
                            crate::diagnostic::SourceLocation::new(0, 0, self.ast.file.clone()),
                        ),
                        Id::Value(id),
                        false,
                    );
                    ir_body.ops.push(var);
                    args.push(Id::Value(id));
                }
                "float" => {
                    let value = name.as_str().parse::<f64>().unwrap();
                    let id = ValueId(ir_body.ops.len());
                    let var_name = format!("raw_arg_{}", id.0);
                    let var = self.assign_local(
                        var_name.clone(),
                        crate::ast::Expression::new(
                            crate::ast::ExpressionKind::Float(value),
                            crate::diagnostic::SourceLocation::new(0, 0, self.ast.file.clone()),
                        ),
                        Id::Value(id),
                        false,
                    );
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
                    let var = self.assign_local(
                        var_name.clone(),
                        crate::ast::Expression::new(
                            crate::ast::ExpressionKind::StringLiteral(value),
                            crate::diagnostic::SourceLocation::new(0, 0, self.ast.file.clone()),
                        ),
                        Id::Value(id),
                        false,
                    );
                    ir_body.ops.push(var);
                    args.push(Id::Value(id));
                }
                _ => {}
            }
        }
        self.symbol_table.pop();
        args
    }

    pub fn lower_element_body(
        &mut self,
        body: &Vec<crate::ast::Statement>,
        hirmodule: &mut HIRModule,
    ) -> FuncBlock {
        let mut ir_body = FuncBlock {
            ops: Vec::new(),
            returned_element_ref: None,
        };

        self.symbol_table.push(HashMap::new()); // add new scope (element)

        for stmt in body {
            match &stmt.node {
                StatementKind::ConstAssign { name, value } => {
                    let id = Id::Value(ValueId(ir_body.ops.len()));
                    let value = self.assign_local(name.clone(), value.clone(), id, false);
                    ir_body.ops.push(value);
                    self.add_symbol(name.clone(), id);
                }
                StatementKind::VarAssign { name, value } => {
                    let id = Id::Value(ValueId(ir_body.ops.len()));
                    let value = self.assign_local(name.clone(), value.clone(), id, true);
                    ir_body.ops.push(value);
                    self.add_symbol(name.clone(), id);
                }
                StatementKind::Return { doc_element } => {
                    let element_id =
                        self.lower_document_element(doc_element, hirmodule, &mut ir_body, None);
                    ir_body.ops.push(Op::Return {
                        doc_element_ref: element_id,
                    });
                    ir_body.returned_element_ref = Some(element_id);
                }
                StatementKind::DocElementEmit { element } => {
                    // Direct element emission without return
                    let element_id =
                        self.lower_document_element(element, hirmodule, &mut ir_body, None);
                    ir_body.ops.push(Op::HirElementEmit { index: element_id });
                    if ir_body.returned_element_ref.is_none() {
                        ir_body.returned_element_ref = Some(element_id);
                    }
                }
                StatementKind::Children { children } => { // TODO for now do nothing and see
                }
                _ => {
                    todo!(
                        "other statement types in element body not handled yet: {:?}",
                        stmt.node
                    )
                }
            }
        }

        self.symbol_table.pop(); // remove scope (element)
        ir_body
    }
}
