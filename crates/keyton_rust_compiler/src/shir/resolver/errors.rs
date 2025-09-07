use crate::span::Span;

#[derive(Debug, Clone)]
pub enum ResolveError {
    UnresolvedName { span: Span, name: String },
    ImportError { span: Span, message: String },
}

#[derive(Debug, Default)]
pub struct ResolveReport {
    pub errors: Vec<ResolveError>,
}
