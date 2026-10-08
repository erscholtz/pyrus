use std::collections::HashMap;

use crate::ast::ContentBlock;
use crate::ast::Inline;
use crate::ast::InlineText;
use crate::diagnostic::Diagnostic;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::Invoke;
use crate::hir::hir_types::TextOp;

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
        let mut text_op = TextOp {
            content: String::new(),
            link_hrefs: Vec::new(),
            bold_ranges: Vec::new(),
            italic_ranges: Vec::new(),
            nerd_font_ranges: Vec::new(),
            link_ranges: Vec::new(),
        };

        let mut len = 0; // NOTE it would be nice if I can do this without that random start var on every case
        for part in &inline.parts {
            let start = len;
            match part {
                Inline::Text(text) => {
                    text_op.content.push_str(&text);
                    len = text_op.content.len();
                }
                Inline::Bold(text) => {
                    text_op.content.push_str(&text);
                    len = text_op.content.len();
                    text_op.bold_ranges.push(start..len);
                }
                Inline::Italic(text) => {
                    text_op.content.push_str(&text);
                    len = text_op.content.len();
                    text_op.italic_ranges.push(start..len);
                }
                Inline::NerdFont(text) => {
                    text_op.content.push_str(&text);
                    len = text_op.content.len();
                    text_op.nerd_font_ranges.push(start..len);
                }
                Inline::Link { label, href } => {
                    text_op.content.push_str(&label);
                    len = text_op.content.len();
                    text_op.link_ranges.push(start..len);
                    text_op.link_hrefs.push(href.clone());
                }
            }
        }

        text_op
    }
}
