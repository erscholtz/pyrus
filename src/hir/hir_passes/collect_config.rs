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
use crate::hir::hir_types::Metadata;

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
    fn find_config(&mut self, config: &DocumentConfig) -> Result<Config, Diagnostic> {
        // define default
        let mut doc_type = DocType::Letter;
        let mut orientation = DocOrientation::Portrait;
        let mut top_margin = 40 as f32;
        let mut bottom_margin = 40 as f32;
        let mut left_margin = 40 as f32;
        let mut right_margin = 40 as f32;
        let mut padding = 0;
        let mut metadata = Metadata::default();

        for entry in &config.entries {
            match entry.name.text.as_str() {
                "doc_type" => match entry.value.node.as_str() {
                    "A4" => {
                        doc_type = DocType::A4;
                    }
                    "Letter" => {
                        doc_type = DocType::Letter;
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
                "title" => metadata.title = Some(entry.value.node.clone()),
                "subtitle" => metadata.subtitle = Some(entry.value.node.clone()),
                "language" => metadata.language = Some(entry.value.node.clone()),
                // comma separated lists, e.g. `author: Ada Lovelace, Alan Turing`
                "author" => metadata.authors.extend(split_list(&entry.value.node)),
                "keywords" => metadata.keywords.extend(split_list(&entry.value.node)),
                // repeatable: one `link:` entry per url
                "link" => metadata.links.push(entry.value.node.clone()),
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
            metadata,
        })
    }
}

fn split_list(value: &str) -> impl Iterator<Item = String> + '_ {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
}
