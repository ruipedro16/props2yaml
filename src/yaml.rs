use std::collections::HashMap;

#[allow(dead_code)]
fn insert_nested(root: &mut serde_yaml::Mapping, key: &str, value: &str) {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.len() == 1 {
        root.insert(
            serde_yaml::Value::String(key.to_string()),
            // first we check if it is a number, and only then write it as a string
            serde_yaml::from_str::<serde_yaml::Value>(value)
                .unwrap_or(serde_yaml::Value::String(value.to_string())),
        );
        return;
    }

    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;
        let key_value = serde_yaml::Value::String(part.to_string());

        if is_last {
            current.insert(
                key_value,
                serde_yaml::from_str::<serde_yaml::Value>(value)
                    .unwrap_or(serde_yaml::Value::String(value.to_string())),
            );
        } else {
            current
                .entry(key_value.clone())
                .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

            current = match current.get_mut(&key_value) {
                Some(serde_yaml::Value::Mapping(m)) => m,
                _ => return, // Should not happen
            };
        }
    }
}

#[allow(dead_code)]
pub(crate) fn write(map: &HashMap<String, String>) -> String {
    let mut root = serde_yaml::Mapping::new();

    for (key, value) in map {
        insert_nested(&mut root, key, value);
    }

    let yaml_value = serde_yaml::Value::Mapping(root);

    // TODO: FIXME: Proper error handling
    serde_yaml::to_string(&yaml_value).unwrap()
}

#[allow(dead_code)]
fn flatten_yaml(value: &serde_yaml::Value, prefix: String, map: &mut HashMap<String, String>) {
    match value {
        serde_yaml::Value::Mapping(m) => {
            for (k, v) in m {
                if let Some(key_str) = k.as_str() {
                    let new_prefix = if prefix.is_empty() {
                        key_str.to_string()
                    } else {
                        format!("{}.{}", prefix, key_str)
                    };
                    flatten_yaml(v, new_prefix, map);
                }
            }
        }

        serde_yaml::Value::Sequence(seq) => {
            for (i, item) in seq.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, i);
                flatten_yaml(item, new_prefix, map);
            }
        }

        serde_yaml::Value::String(s) => {
            map.insert(prefix, s.clone());
        }

        serde_yaml::Value::Number(n) => {
            map.insert(prefix, n.to_string());
        }

        serde_yaml::Value::Bool(b) => {
            map.insert(prefix, b.to_string());
        }

        serde_yaml::Value::Null => {
            map.insert(prefix, String::new());
        }

        _ => {
            panic!("should not happen");
        }
    }
}

#[allow(dead_code)]
pub(crate) fn parse(content: &str) -> HashMap<String, String> {
    let value: serde_yaml::Value = serde_yaml::from_str(content).expect("failed to parse yaml");
    let mut map = HashMap::new();

    flatten_yaml(&value, String::new(), &mut map);

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_yaml() {
        let input = "
key: value
foo: bar
";
        let result = parse(input);
        assert_eq!(result.get("key"), Some(&"value".to_string()));
        assert_eq!(result.get("foo"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_parse_nested_yaml() {
        let input = "
database:
  host: localhost
  port: 5432
";
        let result = parse(input);
        assert_eq!(result.get("database.host"), Some(&"localhost".to_string()));
        assert_eq!(result.get("database.port"), Some(&"5432".to_string()));
    }

    #[test]
    fn test_write_simple() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value".to_string());
        let output = write(&map);
        assert!(output.contains("key:"));
        assert!(output.contains("value"));
        assert!(output.contains("key: value"),);
    }

    #[test]
    fn test_write_nested() {
        let mut map = HashMap::new();
        map.insert("database.host".to_string(), "localhost".to_string());
        map.insert("database.port".to_string(), "5432".to_string());

        let output = write(&map);
        assert!(
            output.contains("database:"),
            "Output missing 'database:' block:\n{}",
            output
        );
        assert!(
            output.contains("host: localhost"),
            "Output missing nested host:\n{}",
            output
        );
        assert!(
            output.contains("port: 5432"),
            "Output missing nested port:\n{}",
            output
        );
    }
}
