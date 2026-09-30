use crate::ast::Ast;
use crate::ast::DocumentConfig;
use crate::ast::Item;
use crate::diagnostic::Diagnostic;
use crate::diagnostic::Severity;
use crate::hir::HIR;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::Config;
use crate::hir::hir_types::DocOrientation;
use crate::hir::hir_types::DocType;

pub struct CollectConfig;

impl HIRPass for CollectConfig {
    fn run(&mut self, hir: &mut HIR, ast: &Ast) -> Result<(), Vec<Diagnostic>> {
        for item in &ast.items {
            match &item.node {
                Item::Document(conf) => {
                    let config = match self.find_config(conf) {
                        Ok(config) => config,
                        Err(err) => return Err(vec![err]), // TODO this is wrong.
                    };
                    hir.config = config;
                    return Ok(());
                }
                _ => {
                    // TODO proper error handling
                    // NOTE think of what to do in case of multiple document
                    // items
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "CollectConfig"
    }
}

impl Default for CollectConfig {
    fn default() -> Self {
        Self
    }
}

impl CollectConfig {
    fn find_config(
        &mut self,
        config: &DocumentConfig,
    ) -> Result<Config, Diagnostic> {
        // define default
        let mut doc_type = DocType::A4;
        let mut orientation = DocOrientation::Portrait;
        let mut top_margin = 0;
        let mut bottom_margin = 0;
        let mut left_margin = 0;
        let mut right_margin = 0;
        let mut padding = 0;

        for entry in &config.entries {
            match entry.name.text.as_str().clone() {
                "doc_type" => match entry.value.node.as_str() {
                    "A4" => {
                        doc_type = DocType::A4;
                    }
                    _ => {
                        // TODO proper error handling here
                        return Err(Diagnostic {
                            severity: Severity::Error,
                            message: "invalid doc type".to_string(),
                            location: entry.value.location.clone(),
                            span: None,
                            help: None,
                        });
                    }
                },
                "orientation" => match entry.value.node.as_str() {
                    "portrait" => {
                        orientation = DocOrientation::Portrait;
                    }
                    _ => {
                        // TODO proper error handling here
                        return Err(Diagnostic {
                            severity: Severity::Error,
                            message: "invalid orientation".to_string(),
                            location: entry.value.location.clone(),
                            span: None,
                            help: None,
                        });
                    }
                },
                "top" => {
                    top_margin = entry.value.node.parse().unwrap();
                }
                "bottom" => {
                    bottom_margin = entry.value.node.parse().unwrap();
                }
                "left" => {
                    left_margin = entry.value.node.parse().unwrap();
                }
                "right" => {
                    right_margin = entry.value.node.parse().unwrap();
                }
                "padding" => {
                    padding = entry.value.node.parse().unwrap();
                }
                _ => {
                    // TODO proper error handling here
                }
            }
        }
        Ok(Config {
            doc_type,
            orientation,
            top_margin,
            bottom_margin,
            left_margin,
            right_margin,
            padding,
        })
    }
}
