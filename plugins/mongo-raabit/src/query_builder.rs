//! Visual Query Builder: filter conditions, sort and projection without writing code.

use crate::style::Palette;
use egui::Ui;
use mongo_raabit_core::bson::Document;
use mongo_raabit_core::query::QueryText;

/// What happened in the builder this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    None,
    Changed,
    Run,
}

/// Builder state of one collection tab.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QueryBuilder {}

impl QueryBuilder {
    /// The query as shell text, as the Code mode shows it.
    pub fn to_query_text(&self) -> QueryText {
        QueryText::default()
    }

    /// Takes over a query written in Code mode when the builder can represent it.
    pub fn take_over(&mut self, _query: &QueryText) {}
}

/// Field paths seen in `documents` (dotted for nested fields).
pub fn field_names(_documents: &[Document]) -> Vec<String> {
    Vec::new()
}

pub fn show(_ui: &mut Ui, _p: &Palette, _builder: &mut QueryBuilder, _fields: &[String]) -> Event {
    Event::None
}
