use std::collections::HashMap;

use crate::ast::ContentBlock;
use crate::ast::Inline;
use crate::ast::InlineText;
use crate::diagnostic::Diagnostic;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::Invoke;
use crate::hir::hir_types::TextOp;
use crate::hir::hir_types::TextType;

pub struct CollectInvokes;

impl HIRPass for CollectInvokes {
    fn run(
        &mut self,
        hir: &mut crate::hir::HIR,
        ast: &crate::ast::Ast,
    ) -> Result<(), Vec<Diagnostic>> {
        for item in &ast.items {
            match &item.node {
                crate::ast::Item::ElemInvoke(invoke) => {
                    let name = invoke.name.text.clone();
                    let mut elements = HashMap::new();
                    for field in &invoke.fields {
                        let name = field.name.text.clone();
                        let text = self.lower_inlines(field.value.clone());

                        elements.insert(name, vec![text]);
                    }

                    if let Some(content) = invoke.content.clone() {
                        let mut content_parts = vec![];
                        for block in &content.blocks {
                            match block {
                                ContentBlock::Paragraph(text) => {
                                    content_parts
                                        .push(self.lower_inlines(text.clone()));
                                }
                                ContentBlock::BulletList(items) => {
                                    for item in items {
                                        content_parts.push(
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
    fn lower_inlines(&self, inline: InlineText) -> TextOp {
        let mut content = Vec::new();
        for part in &inline.parts {
            match part {
                Inline::Text(text) => {
                    content.push(TextType::Text(text.clone()))
                }
                Inline::Bold(text) => {
                    content.push(TextType::Bold(text.clone()))
                }
                Inline::Italic(text) => {
                    content.push(TextType::Italic(text.clone()))
                }
                Inline::NerdFont(text) => {
                    content.push(TextType::Nerd(text.clone()))
                }
                Inline::Link { label, href } => content.push(TextType::Link {
                    label: label.clone(),
                    href: href.clone(),
                }),
            }
        }

        TextOp { content }
    }
}
