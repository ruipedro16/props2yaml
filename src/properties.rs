use std::collections::HashMap;

fn escape_key(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('=', "\\=")
        .replace(':', "\\:")
        .replace(' ', "\\ ")
}

fn escape_value(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

pub fn write(map: &HashMap<String, String>) -> String {
    let mut lines: Vec<String> = map
        .iter()
        .map(|(k, v)| format!("{}={}", escape_key(k), escape_value(v)))
        .collect();

    lines.sort();
    lines.join("\n") + "\n"
}

fn unescape_value(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            match ch {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                _ => {
                    result.push('\\');
                    result.push(ch);
                }
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            result.push(ch);
        }
    }

    result
}

fn parse_property_line(line: &str) -> Option<(String, String)> {
    let mut key = String::new();
    let mut chars = line.chars().peekable();
    let mut escaped = false;

    // Parse the key
    while let Some(ch) = chars.next() {
        if escaped {
            key.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '=' || ch == ':' || ch.is_whitespace() {
            // Skip any additional whitespace or separators
            while chars
                .peek()
                .map_or(false, |c| c.is_whitespace() || *c == '=' || *c == ':')
            {
                chars.next();
            }
            break;
        }

        key.push(ch);
    }

    let value: String = chars.collect();
    let value = unescape_value(&value);

    if key.is_empty() {
        return None;
    }

    Some((key.trim().to_string(), value))
}

pub fn parse(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for (_, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }

        // Find the separator (=, :, or whitespace)
        // TODO: use InvalidPropertyLine
        let (key, value) = parse_property_line(trimmed).unwrap();

        /*
           .ok_or_else(|| {
               Prop2YamlError::InvalidPropertyLine(
                   format!("Line {}: '{}'", line_num + 1, line)
               )
           });

        */

        map.insert(key, value);
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value".to_string());
        let output = write(&map);
        assert!(output.contains("key=value"));
    }

    #[test]
    fn test_parse_simple() {
        let input = "key=value\nfoo=bar";
        let result = parse(input);
        assert_eq!(result.get("key"), Some(&"value".to_string()));
        assert_eq!(result.get("foo"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_parse_with_comments() {
        let input = "# Comment\nkey=value\n! Another comment\nfoo=bar";
        let result = parse(input);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_with_colons() {
        let input = "key:value";
        let result = parse(input);
        assert_eq!(result.get("key"), Some(&"value".to_string()));
    }
}
