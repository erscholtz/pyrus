pub mod hir_passes;
pub mod hir_types;
pub mod hir_util;

use crate::ast::Ast;
use crate::diagnostic::CompilerDiagnostic;
use crate::hir::hir_passes::collect_invokes::CollectInvokes;
use crate::hir::hir_passes::collect_layouts::CollectLayouts;
use crate::hir::{
    hir_passes::{PassManager, collect_decls::CollectDecls},
    hir_types::HIR,
};

pub fn lower(ast: &Ast) -> Result<HIR, Vec<CompilerDiagnostic>> {
    let mut hir = HIR::new(&ast.file);

    let result = PassManager::default()
        .continue_on_error()
        .run::<CollectDecls>(&mut hir, ast) // global variables
        .run::<CollectLayouts>(&mut hir, ast) // function declarations
        .run::<CollectInvokes>(&mut hir, ast) // document elements
        // .run::<ValidationPass>(&mut hir, ast) // validation checks
        // .run::<DeadCodeElim>(&mut hir, ast) // Dead code removal
        .finished();

    if let Err(errors) = result {
        Err(errors.into_iter().map(|e| e.into()).collect()) // crazy conversion
    } else {
        Ok(hir)
    }
}
