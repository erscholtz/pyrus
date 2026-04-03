use crate::ast::{KeyValue, Selector, StyleRule};
use crate::hir::ir_types::{HIRModule, PageBreak, StyleAttributes};

pub fn resolve_styles(hir: &mut HIRModule) {
    let mut resolver = StyleResolver::new(hir);
    resolver.resolve();
}

pub struct StyleResolver<'a> {
    hir: &'a mut HIRModule,
}

impl<'a> StyleResolver<'a> {
    pub fn new(hir: &'a mut HIRModule) -> Self {
        Self { hir }
    }

    pub fn resolve(&mut self) {
        // Sort CSS rules by specificity for cascade order
        let mut sorted_rules = self.hir.css_rules.clone();
        sorted_rules.sort_by_key(|r| r.specificity);

        for element_idx in 0..self.hir.element_metadata.len() {
            self.compute_element_styles(element_idx, &sorted_rules);
        }
    }

    fn compute_element_styles(&mut self, element_idx: usize, sorted_rules: &[StyleRule]) {
        let metadata = &self.hir.element_metadata[element_idx].clone();

        // Start with inherited styles from parent
        let mut computed = StyleAttributes::default();
        if let Some(parent_idx) = metadata.parent {
            self.apply_inherited_styles(&mut computed, parent_idx);
        }

        for rule in sorted_rules {
            if self.rule_matches(rule, element_idx) {
                self.apply_rule_declarations(&mut computed, &rule.declaration_block);
            }
        }

        self.apply_inline_styles(&mut computed, metadata.attributes_ref);
        self.update_computed_styles(metadata.attributes_ref, computed);
    }

    fn apply_inherited_styles(&self, computed: &mut StyleAttributes, parent_idx: usize) {
        let parent_metadata = &self.hir.element_metadata[parent_idx];

        if let Some(parent_node) = self
            .hir
            .attributes
            .find_node(parent_metadata.attributes_ref)
        {
            computed.apply_inherited(&parent_node.computed);
        }
    }

    fn rule_matches(&self, rule: &StyleRule, element_idx: usize) -> bool {
        // A rule matches if ANY of its selectors match the element
        rule.selector_list
            .iter()
            .any(|selector| self.selector_matches(selector, element_idx))
    }

    fn selector_matches(&self, selector: &Selector, element_idx: usize) -> bool {
        let metadata = &self.hir.element_metadata[element_idx];

        match selector {
            Selector::Id(id) => metadata.id.as_ref() == Some(id),
            Selector::Class(class) => metadata.classes.contains(class),
            Selector::Type(ty) => metadata.element_type == *ty,
        }
    }

    fn apply_rule_declarations(
        &mut self,
        computed: &mut StyleAttributes,
        declarations: &[KeyValue],
    ) {
        for decl in declarations {
            let value_str = self.expr_to_string(&decl.value);
            computed.set(&decl.key, value_str);
        }
    }

    /// Inline styles override CSS rules (higher specificity)
    fn apply_inline_styles(&self, computed: &mut StyleAttributes, attributes_ref: usize) {
        if let Some(node) = self.hir.attributes.find_node(attributes_ref) {
            let inline = &node.inline;

            // Merge inline styles into computed, with inline taking precedence
            // This handles the inline style="..." attribute
            for (key, val) in &inline.style {
                computed.set(key, val.clone());
            }

            // Also apply known attributes as inline styles
            if let Some(id) = &inline.id {
                computed.id = Some(id.clone());
            }
            if !inline.class.is_empty() {
                computed.class = inline.class.clone();
            }
            if let Some(margin) = inline.margin {
                computed.margin = Some(margin);
            }
            if let Some(padding) = inline.padding {
                computed.padding = Some(padding);
            }
            if let Some(align) = &inline.align {
                computed.align = Some(align.clone());
            }
            if inline.hidden {
                computed.hidden = true;
            }
            if inline.page_break != PageBreak::None {
                computed.page_break = inline.page_break.clone();
            }
            if let Some(role) = &inline.role {
                computed.role = Some(role.clone());
            }
        }
    }

    fn update_computed_styles(&mut self, attributes_ref: usize, computed: StyleAttributes) {
        if let Some(node) = self.hir.attributes.find_node_mut(attributes_ref) {
            node.computed = computed;
        }
    }

    fn expr_to_string(&mut self, expr: &crate::ast::Expression) -> String {
        use crate::ast::ExpressionKind;

        match &expr.node {
            ExpressionKind::StringLiteral(s) => s.clone(),
            ExpressionKind::Int(n) => n.to_string(),
            ExpressionKind::Float(f) => f.to_string(),
            ExpressionKind::Identifier(s) => s.clone(),
            ExpressionKind::StructDefault(s) => format!("default({})", s),
            ExpressionKind::InterpolatedString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        crate::ast::InterpPart::Text(text) => result.push_str(text),
                        crate::ast::InterpPart::Expression(expr_kind) => {
                            // Create a temporary Expression wrapper for the ExpressionKind
                            let temp_expr = crate::ast::Expression::new(
                                expr_kind.clone(),
                                crate::error::SourceLocation::new(0, 0, self.hir.file.clone()),
                            );
                            result.push_str(&self.expr_to_string(&temp_expr))
                        }
                    }
                }
                result
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {:?} {}",
                    self.expr_to_string(left),
                    operator,
                    self.expr_to_string(right)
                )
            }
            ExpressionKind::Unary {
                operator,
                expression,
            } => {
                format!("{:?} {}", operator, self.expr_to_string(expression))
            }
        }
    }
}
