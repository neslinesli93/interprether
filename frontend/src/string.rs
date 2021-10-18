pub struct StringPart {
    pub s: String,
    pub t: StringPartType,
}

#[derive(PartialEq)]
pub enum StringPartType {
    Normal,
    Highlight,
}

pub fn split_keep<'a>(text: &'a str, r: &'a str) -> Vec<StringPart> {
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in text.to_lowercase().match_indices(&r) {
        if last != index {
            result.push(StringPart {
                s: text[last..index].to_string(),
                t: StringPartType::Normal,
            });
        }

        last = index + matched.len();
        result.push(StringPart {
            s: text[index..last].to_string(),
            t: StringPartType::Highlight,
        });
    }

    if last < text.len() {
        result.push(StringPart {
            s: text[last..].to_string(),
            t: StringPartType::Normal,
        });
    }

    result
}
