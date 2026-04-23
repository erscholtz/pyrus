use crate::ast::{Ast, KeyValue, Selector, StyleRule};
use crate::hir::HIRModule;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::{PageBreak, StyleAttributes};
use crate::hir::hir_util::hir_error::HirError;

pub struct StylePass;

impl<'ast_lifetime> HIRPass for StylePass {
    fn run(&mut self, hir: &mut crate::hir::HIRModule, ast: &Ast) -> Result<(), Vec<HirError>> {
        // Store CSS rules from AST
        if let Some(style) = ast.style.clone() {
            hir.css_rules = style.statements.clone();
        }
        self.resolve(hir);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "style_pass"
    }
}

impl Default for StylePass {
    fn default() -> Self {
        Self {}
    }
}

impl StylePass {
    fn resolve(&mut self, hir: &mut HIRModule) {
        // Sort CSS rules by specificity for cascade order

        let mut sorted_rules = hir.css_rules.clone();
        sorted_rules.sort_by_key(|r| r.specificity);

        for element_idx in 0..hir.element_metadata.len() {
            self.compute_element_styles(element_idx, &sorted_rules, hir);
        }
    }

    fn compute_element_styles(
        &mut self,
        element_idx: usize,
        sorted_rules: &[StyleRule],
        hir: &mut HIRModule,
    ) {
        let metadata = hir.element_metadata[element_idx].clone();

        // Start with inherited styles from parent
        let mut computed = StyleAttributes::default();
        if let Some(parent_idx) = metadata.parent {
            self.apply_inherited_styles(&mut computed, parent_idx, hir);
        }

        for rule in sorted_rules {
            if self.rule_matches(rule, element_idx, hir) {
                self.apply_rule_declarations(&mut computed, &rule.declaration_block);
            }
        }

        self.apply_inline_styles(&mut computed, metadata.attributes_ref, hir);
        self.update_computed_styles(metadata.attributes_ref, computed, hir);
    }

    fn apply_inherited_styles(
        &self,
        computed: &mut StyleAttributes,
        parent_idx: usize,
        hir: &HIRModule,
    ) {
        let parent_metadata = &hir.element_metadata[parent_idx];

        if let Some(parent_node) = hir.attributes.find_node(parent_metadata.attributes_ref) {
            computed.apply_inherited(&parent_node.computed);
        }
    }

    fn rule_matches(&self, rule: &StyleRule, element_idx: usize, hir: &HIRModule) -> bool {
        // A rule matches if ANY of its selectors match the element
        rule.selector_list
            .iter()
            .any(|selector| self.selector_matches(selector, element_idx, hir))
    }

    fn selector_matches(&self, selector: &Selector, element_idx: usize, hir: &HIRModule) -> bool {
        let metadata = hir.element_metadata[element_idx].clone();

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
            let value_str = self.style_value_to_string(&decl.value);
            computed.set(&decl.key, value_str);
        }
    }

    /// Inline styles override CSS rules (higher specificity)
    fn apply_inline_styles(
        &self,
        computed: &mut StyleAttributes,
        attributes_ref: usize,
        hir: &HIRModule,
    ) {
        if let Some(node) = hir.attributes.find_node(attributes_ref) {
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

    fn update_computed_styles(
        &mut self,
        attributes_ref: usize,
        computed: StyleAttributes,
        hir: &mut HIRModule,
    ) {
        if let Some(node) = hir.attributes.find_node_mut(attributes_ref) {
            node.computed = computed;
        }
    }

    fn style_value_to_string(&self, value: &crate::ast::StyleValue) -> String {
        let mut rendered = self.expr_to_string(&value.expr);
        if let Some(unit) = &value.unit {
            rendered.push_str(unit);
        }
        rendered
    }

    fn expr_to_string(&self, expr: &crate::ast::Expr) -> String {
        self.expr_kind_to_string(&expr.node)
    }

    fn expr_kind_to_string(&self, expr: &crate::ast::ExprKind) -> String {
        use crate::ast::{BinaryExpr, ExprKind, InterpolatedStringExpr, UnaryExpr};

        match expr {
            ExprKind::StringLiteral(s) => s.clone(),
            ExprKind::Int(n) => n.to_string(),
            ExprKind::Float(f) => f.to_string(),
            ExprKind::Identifier(s) => s.clone(),
            ExprKind::StructDefault(s) => format!("default({})", s.name),
            ExprKind::InterpolatedString(InterpolatedStringExpr { parts }) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        ExprKind::StringLiteral(s) => result.push_str(s),
                        ExprKind::Int(n) => result.push_str(&n.to_string()),
                        ExprKind::Float(f) => result.push_str(&f.to_string()),
                        ExprKind::Identifier(s) => result.push_str(s),
                        ExprKind::StructDefault(s) => {
                            result.push_str(&format!("default({})", s.name))
                        }
                        _ => {}
                    }
                }
                result
            }
            ExprKind::Binary(BinaryExpr { left, op, right }) => {
                format!(
                    "{} {:?} {}",
                    self.expr_kind_to_string(left),
                    op,
                    self.expr_kind_to_string(right)
                )
            }
            ExprKind::Unary(UnaryExpr { op, expr }) => {
                format!("{:?} {}", op, self.expr_kind_to_string(expr))
            }
        }
    }
}
