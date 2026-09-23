use std::collections::HashMap;

use crate::ast::ContentBlock;
use crate::ast::Inline;
use crate::ast::InlineText;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::Content;
use crate::hir::hir_types::Invoke;

pub struct CollectInvokes;

impl HIRPass for CollectInvokes {
    fn run(
        &mut self,
        hir: &mut crate::hir::HIR,
        ast: &crate::ast::Ast,
    ) -> Result<(), Vec<crate::diagnostic::CompilerDiagnostic>> {
        for item in &ast.items {
            match &item.node {
                crate::ast::Item::ElemInvoke(invoke) => {
                    let name = invoke.name.text.clone();
                    let mut elements = HashMap::new();
                    for field in &invoke.fields {
                        let name = field.name.text.clone();
                        let parts = self.lower_inlines(field.value.clone());

                        elements.insert(name, parts);
                    }

                    if let Some(content) = invoke.content.clone() {
                        let mut content_parts = vec![];
                        for block in &content.blocks {
                            match block {
                                ContentBlock::Paragraph(text) => {
                                    content_parts.extend(
                                        self.lower_inlines(text.clone()),
                                    );
                                }
                                ContentBlock::BulletList(items) => {
                                    for item in items {
                                        content_parts.extend(
                                            self.lower_inlines(item.clone()),
                                        );
                                    }
                                }
                            }
                        }
                        elements.insert("content".to_string(), content_parts);
                    }

                    hir.invokes.push(Invoke { name, elements });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "CollectInvokes"
    }
}

impl Default for CollectInvokes {
    fn default() -> Self {
        Self {}
    }
}

impl CollectInvokes {
    fn lower_inlines(&self, inline: InlineText) -> Vec<Content> {
        let mut parts = Vec::new();
        for part in &inline.parts {
            match part {
                Inline::Text(text) => parts.push(Content::Text(text.clone())),
                Inline::Bold(text) => parts.push(Content::Bold(text.clone())),
                Inline::Italic(text) => {
                    parts.push(Content::Italic(text.clone()))
                }
                Inline::NerdFont(text) => {
                    parts.push(Content::NerdFont(text.clone()))
                }
                Inline::Link { label, href } => parts.push(Content::Link {
                    label: label.clone(),
                    href: href.clone(),
                }),
            }
        }

        parts
    }
}
