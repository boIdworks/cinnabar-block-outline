//! The Hive menu the run walks through and its buttons.

use super::*;

/// A Hive button menu with its text cleaned of formatting.
pub(super) struct Menu {
    pub(super) title: String,
    pub(super) content: String,
    pub(super) buttons: Vec<Button>,
    /// A custom form with a text box (Hive's username field).
    pub(super) text_input: bool,
}

/// A button's first line (its name), second line (its detail) and picture.
pub(super) struct Button {
    pub(super) first: String,
    pub(super) second: String,
    pub(super) image: Option<String>,
}

impl Menu {
    pub(super) fn new(value: &Value) -> Self {
        let text = |value: Option<&Value>| strip(value.and_then(Value::as_str).unwrap_or(""));
        let buttons = value
            .get("buttons")
            .and_then(Value::as_array)
            .map(|buttons| {
                buttons
                    .iter()
                    .map(|button| {
                        let label = text(button.get("text"));
                        let mut lines = label.lines();
                        Button {
                            first: lines.next().unwrap_or("").trim().to_owned(),
                            second: lines.next().unwrap_or("").trim().to_owned(),
                            image: button
                                .pointer("/image/data")
                                .and_then(Value::as_str)
                                .map(str::to_owned),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        let text_input = value.get("type").and_then(Value::as_str) == Some("custom_form")
            && value
                .get("content")
                .and_then(Value::as_array)
                .is_some_and(|elements| {
                    elements
                        .iter()
                        .any(|element| element.get("type").and_then(Value::as_str) == Some("input"))
                });
        Self {
            title: text(value.get("title")),
            content: text(value.get("content")),
            buttons,
            text_input,
        }
    }

    /// The button whose name is `name`, ignoring case.
    pub(super) fn button(&self, name: &str) -> Option<usize> {
        self.buttons
            .iter()
            .position(|button| button.first.eq_ignore_ascii_case(name))
    }

    /// The first button whose name has one of the `wanted` words and none of the `unwanted`.
    pub(super) fn button_with(&self, wanted: &[&str], unwanted: &[&str]) -> Option<usize> {
        self.buttons.iter().position(|button| {
            let name = button.first.to_lowercase();
            wanted.iter().any(|word| name.contains(word))
                && !unwanted.iter().any(|word| name.contains(word))
        })
    }
}
