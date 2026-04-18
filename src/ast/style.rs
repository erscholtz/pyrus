use crate::ast::Expr;

#[derive(Debug, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: StyleValue,
}

#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector_list: Vec<Selector>,
    pub declaration_block: Vec<KeyValue>,
    pub specificity: usize, // Pre-computed specificity for cascade ordering
}

impl StyleRule {
    pub fn new(selector_list: Vec<Selector>, declaration_block: Vec<KeyValue>) -> Self {
        let specificity = Self::compute_specificity(&selector_list);
        Self {
            selector_list,
            declaration_block,
            specificity,
        }
    }

    /// Compute CSS specificity: (id_count, class_count, type_count) as a single number
    /// Returns: id_count * 100 + class_count * 10 + type_count
    fn compute_specificity(selectors: &[Selector]) -> usize {
        let mut id_count = 0;
        let mut class_count = 0;
        let mut type_count = 0;

        for selector in selectors {
            match selector {
                Selector::Id(_) => id_count += 1,
                Selector::Class(_) => class_count += 1,
                Selector::Type(_) => type_count += 1,
            }
        }

        id_count * 100 + class_count * 10 + type_count
    }
}

#[derive(Debug, Clone)]
pub struct StyleValue {
    pub expr: Expr,
    pub unit: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Selector {
    Class(String),
    Id(String),
    Type(String),
}
