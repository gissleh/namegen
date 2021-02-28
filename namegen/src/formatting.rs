use crate::core::WorkingSet;

#[derive(Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum FormattingRule {
    CapitalizeFirst,
    CapitalizeDefault,
    CapitalizeAfter(char),
    RemoveChar(char),
    ReplaceChar{from: char, to: char},
}

/// Format WorkingSet's content.
pub fn format_ws(ws: &mut WorkingSet, rules: &[FormattingRule]) {
    ws.result_chars.clear();
    ws.result_chars.extend(ws.result_str.chars());

    format_vec(&mut ws.result_chars, rules);

    ws.result_str.clear();
    ws.result_str.extend(ws.result_chars.iter());
}

/// Format string. This does two allocations.
#[allow(dead_code)]
pub fn format_string(s: &str, rules: &[FormattingRule]) -> String {
    let mut chars = s.chars().collect();
    format_vec(&mut chars, rules);

    chars.iter().collect()
}

/// Format the content of the vector with the given rules. This
/// does not allocate.
pub fn format_vec(v: &mut Vec<char>, rules: &[FormattingRule]) {
    if v.len() == 0 || rules.len() == 0 {
        return;
    }

    // Capitalization rules
    let mut capitalized_first = false;
    for rule in rules.iter() {
        match *rule {
            FormattingRule::CapitalizeFirst => {
                capitalize(v, 0);

                capitalized_first = true;
            }

            FormattingRule::CapitalizeDefault => {
                let mut i = if capitalized_first { 1 } else { 0 };
                while i < v.len() {
                    i += 1 + capitalize(v, i);
                }
            }

            FormattingRule::CapitalizeAfter(ch) => {
                let mut i = 1;
                while i < v.len() {
                    if v[i - 1] == ch {
                        i += 1 + capitalize(v, i);
                    } else {
                        i += 1;
                    }
                }
            }

            _ => {}
        }
    }

    // Last rules
    for rule in rules.iter() {
        match *rule {
            FormattingRule::ReplaceChar {from, to} => {
                for i in 0..v.len() {
                    if v[i] == from {
                        v[i] = to;
                    }
                }
            }

            FormattingRule::RemoveChar (ch) => {
                let mut i = 0;
                while i < v.len() {
                    if v[i] == ch {
                        v.remove(i);
                    } else {
                        i += 1;
                    }
                }
            }

            _ => {}
        }
    }
}

fn capitalize(v: &mut Vec<char>, offset: usize) -> usize {
    let mut upper = v[offset].to_uppercase();
    let mut extra = 0;
    if let Some(c) = upper.next() {
        v[offset] = c;
    }
    for c in upper {
        extra += 1;
        v.insert(offset + extra, c);
    }

    extra
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_rules() {
        assert_eq!(format_string("hello", &[FormattingRule::CapitalizeDefault]), "HELLO");
        assert_eq!(format_string("hello", &[FormattingRule::CapitalizeFirst]), "Hello");
        assert_eq!(format_string("hello", &[FormattingRule::RemoveChar('l')]), "heo");
        assert_eq!(format_string("hello_world", &[FormattingRule::ReplaceChar{from: '_', to: ' '}]), "hello world");
        assert_eq!(format_string("hello_world", &[FormattingRule::CapitalizeAfter('_')]), "hello_World");

        // Test case for the one-to-multi capitalization.
        assert_eq!(format_string("straße", &[FormattingRule::CapitalizeDefault]), "STRASSE");
    }

    #[test]
    fn test_multiple_capitalization() {
        assert_eq!(
            // Mass Effect asari use case
            format_string("t'soni", &[
                FormattingRule::CapitalizeFirst,
                FormattingRule::CapitalizeAfter('\''),
            ]),

            "T'Soni"
        );

        assert_eq!(
            // Elder Scrolls Breton use case
            format_string("du_bois", &[
                FormattingRule::CapitalizeFirst,
                FormattingRule::CapitalizeAfter('_'),
                FormattingRule::RemoveChar('_'),
            ]),

            "DuBois"
        );

        assert_eq!(
            // Mass Effect salarian use case
            format_string("jan_mayr", &[
                FormattingRule::CapitalizeFirst,
                FormattingRule::CapitalizeAfter('_'),
                FormattingRule::ReplaceChar{from: '_', to: ' '},
            ]),

            "Jan Mayr"
        );
    }
}