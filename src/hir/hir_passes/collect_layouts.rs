use std::collections::HashMap;

use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::Layout;
use crate::hir::hir_types::Sizing;

pub struct CollectLayouts;

impl HIRPass for CollectLayouts {
    fn run(
        &mut self,
        hir: &mut crate::hir::HIR,
        ast: &crate::ast::Ast,
    ) -> Result<(), Vec<crate::diagnostic::CompilerDiagnostic>> {
        let diagnostics = Vec::new();
        for item in &ast.items {
            match &item.node {
                crate::ast::Item::LayoutDecl(layout) => {
                    let name = layout.element.text.clone();
                    let mut sizing = HashMap::new();

                    for prop in &layout.props {
                        let size = match prop.value.text.as_str() {
                            "xl" => Sizing::Xl,
                            "lg" => Sizing::Lg,
                            "md" => Sizing::Md,
                            "sm" => Sizing::Sm,
                            _ => {
                                // TODO this needs to be implemented correctly
                                // diagnostics.push(
                                //     SemanticError::InvalidStyleProperty {
                                //         location: prop.value.span.clone(),
                                //         property: prop.field.text.clone(),
                                //         value: prop.value.text.clone(),
                                //     },
                                // );
                                continue;
                            }
                        };
                        sizing.insert(prop.field.text.clone(), size);
                    }
                    let layout = Layout {
                        layouts: layout.rows.clone(),
                        sizing: sizing,
                    };

                    hir.layout.insert(name, layout);
                    if !diagnostics.is_empty() {
                        return Err(diagnostics);
                    }
                }
                _ => {}
            }
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "CollectLayouts"
    }
}

impl Default for CollectLayouts {
    fn default() -> Self {
        Self {}
    }
}
