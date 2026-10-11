//! egui renderer for declarative [`ViewNode`] trees.

use crate::{empty_state, property, secondary_text, section};
use egui::{RichText, Ui};
use jetthing_plugin_api::{TreeNode, Value, ViewEvent, ViewNode};

/// Renders `node`; `scope` keeps widget ids unique per view. Returns the
/// interactions of this frame.
pub fn render_node(ui: &mut Ui, scope: &str, node: &ViewNode) -> Vec<ViewEvent> {
    let mut events = Vec::new();
    node_ui(ui, scope, node, &mut events);
    events
}

fn node_ui(ui: &mut Ui, scope: &str, node: &ViewNode, events: &mut Vec<ViewEvent>) {
    let id = |widget: &str| egui::Id::new(("plugin_view", scope, widget.to_owned()));
    match node {
        ViewNode::Column { children } => {
            ui.vertical(|ui| {
                for child in children {
                    node_ui(ui, scope, child, events);
                }
            });
        }
        ViewNode::Row { children } => {
            ui.horizontal(|ui| {
                for child in children {
                    node_ui(ui, scope, child, events);
                }
            });
        }
        ViewNode::Heading { text } => {
            ui.label(RichText::new(text).strong().size(15.0));
        }
        ViewNode::Label { text, muted } => {
            if *muted {
                let color = secondary_text(ui);
                ui.label(RichText::new(text).color(color));
            } else {
                ui.label(text);
            }
        }
        ViewNode::Section { text } => section(ui, text),
        ViewNode::Property { key, value } => property(ui, key, value),
        ViewNode::Separator => {
            ui.separator();
        }
        ViewNode::Button { id: button, label } => {
            if ui.button(label).clicked() {
                events.push(ViewEvent::Clicked { id: button.clone() });
            }
        }
        ViewNode::TextInput {
            id: input,
            value,
            hint,
            multiline,
        } => {
            let mut text = value.clone();
            let edit = if *multiline {
                egui::TextEdit::multiline(&mut text)
            } else {
                egui::TextEdit::singleline(&mut text)
            };
            let response = ui.add(
                edit.id(id(input))
                    .hint_text(hint.as_str())
                    .desired_width(crate::input_width(ui, *multiline)),
            );
            if response.changed() {
                events.push(ViewEvent::Changed {
                    id: input.clone(),
                    value: Value::from(text),
                });
            }
        }
        ViewNode::Checkbox {
            id: checkbox,
            label,
            checked,
        } => {
            let mut value = *checked;
            if ui.checkbox(&mut value, label.as_str()).changed() {
                events.push(ViewEvent::Changed {
                    id: checkbox.clone(),
                    value: Value::from(value),
                });
            }
        }
        ViewNode::Choice {
            id: choice,
            options,
            selected,
        } => {
            egui::ComboBox::from_id_salt(id(choice))
                .selected_text(selected.as_str())
                .show_ui(ui, |ui| {
                    for option in options {
                        if ui.selectable_label(option == selected, option).clicked() {
                            events.push(ViewEvent::Changed {
                                id: choice.clone(),
                                value: Value::from(option.clone()),
                            });
                        }
                    }
                });
        }
        ViewNode::List {
            id: list,
            items,
            selected,
        } => {
            for item in items {
                let is_selected = selected.as_deref() == Some(item.id.as_str());
                let text = match &item.detail {
                    Some(detail) => format!("{}    {detail}", item.label),
                    None => item.label.clone(),
                };
                if ui.selectable_label(is_selected, text).clicked() {
                    events.push(ViewEvent::Selected {
                        id: list.clone(),
                        item: Value::from(item.id.clone()),
                    });
                }
            }
        }
        ViewNode::Table {
            id: table,
            columns,
            rows,
            selected,
        } => {
            egui::Grid::new(id(table))
                .striped(true)
                .num_columns(columns.len().max(1))
                .show(ui, |ui| {
                    for column in columns {
                        ui.label(RichText::new(column).strong());
                    }
                    ui.end_row();
                    for (index, row) in rows.iter().enumerate() {
                        for (column, cell) in row.iter().enumerate() {
                            let response = if column == 0 {
                                ui.selectable_label(*selected == Some(index), cell)
                            } else {
                                ui.label(cell)
                            };
                            if response.clicked() {
                                events.push(ViewEvent::Selected {
                                    id: table.clone(),
                                    item: Value::from(index),
                                });
                            }
                        }
                        ui.end_row();
                    }
                });
        }
        ViewNode::Tree {
            id: tree,
            nodes,
            selected,
        } => {
            for node in nodes {
                tree_node(ui, scope, tree, node, selected.as_deref(), events);
            }
        }
        ViewNode::Code { text } => {
            let mut text = text.as_str();
            ui.add(
                egui::TextEdit::multiline(&mut text)
                    .code_editor()
                    .desired_width(f32::INFINITY),
            );
        }
        ViewNode::Empty { title, hint } => empty_state(ui, title, hint),
    }
}

fn tree_node(
    ui: &mut Ui,
    scope: &str,
    tree: &str,
    node: &TreeNode,
    selected: Option<&str>,
    events: &mut Vec<ViewEvent>,
) {
    let is_selected = selected == Some(node.id.as_str());
    if node.children.is_empty() {
        if ui.selectable_label(is_selected, &node.label).clicked() {
            events.push(ViewEvent::Selected {
                id: tree.to_owned(),
                item: Value::from(node.id.clone()),
            });
        }
        return;
    }
    let header = egui::CollapsingHeader::new(&node.label)
        .id_salt(("plugin_tree", scope, tree, &node.id))
        .show(ui, |ui| {
            for child in &node.children {
                tree_node(ui, scope, tree, child, selected, events);
            }
        });
    if header.header_response.clicked() {
        events.push(ViewEvent::Selected {
            id: tree.to_owned(),
            item: Value::from(node.id.clone()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetthing_plugin_api::ListItem;

    #[test]
    fn renders_every_node_kind_headlessly() {
        let node = ViewNode::Column {
            children: vec![
                ViewNode::Heading { text: "H".into() },
                ViewNode::Row {
                    children: vec![
                        ViewNode::Button {
                            id: "b".into(),
                            label: "Go".into(),
                        },
                        ViewNode::Checkbox {
                            id: "c".into(),
                            label: "On".into(),
                            checked: true,
                        },
                    ],
                },
                ViewNode::TextInput {
                    id: "t".into(),
                    value: "v".into(),
                    hint: String::new(),
                    multiline: false,
                },
                ViewNode::Choice {
                    id: "ch".into(),
                    options: vec!["a".into()],
                    selected: "a".into(),
                },
                ViewNode::List {
                    id: "l".into(),
                    items: vec![ListItem {
                        id: "1".into(),
                        label: "One".into(),
                        detail: None,
                    }],
                    selected: Some("1".into()),
                },
                ViewNode::Table {
                    id: "tb".into(),
                    columns: vec!["A".into()],
                    rows: vec![vec!["x".into()]],
                    selected: None,
                },
                ViewNode::Tree {
                    id: "tr".into(),
                    nodes: vec![TreeNode {
                        id: "r".into(),
                        label: "Root".into(),
                        children: vec![],
                    }],
                    selected: None,
                },
                ViewNode::Code { text: "fn".into() },
                ViewNode::Property {
                    key: "k".into(),
                    value: "v".into(),
                },
                ViewNode::Section { text: "s".into() },
                ViewNode::Label {
                    text: "l".into(),
                    muted: true,
                },
                ViewNode::Separator,
                ViewNode::Empty {
                    title: "e".into(),
                    hint: String::new(),
                },
            ],
        };
        let ctx = egui::Context::default();
        let mut events = Vec::new();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                events = render_node(ui, "test", &node);
            });
        });
        assert!(events.is_empty(), "no input, no events");
    }
}
