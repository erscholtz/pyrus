#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    pub blocks: Vec<ContentBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentBlock {
    Paragraph(InlineText),
    BulletList(Vec<InlineText>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineText {
    pub parts: Vec<Inline>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Bold(String),
    Italic(String),
    NerdFont(String),
    Link { label: String, href: String },
}
