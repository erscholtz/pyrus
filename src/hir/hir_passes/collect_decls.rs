use crate::ast::Ast;
use crate::ast::Item;
use crate::diagnostic::CompilerDiagnostic;
use crate::hir::HIR;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::Elem;

pub struct CollectDecls;

impl HIRPass for CollectDecls {
    fn run(
        &mut self,
        hir: &mut HIR,
        ast: &Ast,
    ) -> Result<(), Vec<CompilerDiagnostic>> {
        let items = ast.items.clone();
        for item in items {
            match item.node {
                Item::ElemDecl(decl) => {
                    let name = decl.name.text.clone();
                    let mut fields: Vec<String> =
                        decl.fields.into_iter().map(|f| f.text).collect();
                    if decl.content {
                        fields.push("content".to_string());
                    }
                    let elem = Elem { elements: fields };
                    hir.decls.insert(name, elem);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "CollectDecls"
    }
}

impl Default for CollectDecls {
    fn default() -> Self {
        Self
    }
}
