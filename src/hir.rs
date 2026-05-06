pub mod hir_debug;
pub mod hir_passes;
pub mod hir_types;
pub mod hir_util;

use std::collections::HashMap;

use crate::ast::Ast;
use crate::diagnostic::{CompilerDiagnostic, DiagnosticManager};
use crate::hir::{
    hir_passes::{
        PassManager, document_pass::DocumentPass, func_pass::FuncPass, global_pass::GlobalPass,
        style_pass::StylePass, validation_pass::ValidationPass,
    },
    hir_types::AttributeTree,
    hir_types::HIRModule,
};

pub fn lower(ast: &Ast, dm: &mut DiagnosticManager) -> Result<HIRModule, Vec<CompilerDiagnostic>> {
    let mut hirmodule = HIRModule {
        file: ast.file.clone(),
        globals: HashMap::new(),
        functions: HashMap::new(),
        element_decls: HashMap::new(),
        attributes: AttributeTree::new(),
        css_rules: Vec::new(),
        elements: Vec::new(),
        element_metadata: Vec::new(),
    };

    let result = PassManager::default()
        .continue_on_error()
        .run::<GlobalPass>(&mut hirmodule, ast) // global variables
        .run::<FuncPass>(&mut hirmodule, ast) // function declarations
        .run::<DocumentPass>(&mut hirmodule, ast) // document elements
        .run::<StylePass>(&mut hirmodule, ast) // css styling
        .run::<ValidationPass>(&mut hirmodule, ast) // validation checks
        .finished();

    if let Err(errors) = result {
        Err(errors.into_iter().map(|e| e.into()).collect()) // crazy conversion
    } else {
        Ok(hirmodule)
    }
}
