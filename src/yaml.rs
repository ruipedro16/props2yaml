use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

pub fn write(
    map: &IndexMap<String, String>,
    skip_format: bool,
    verbose: bool,
    yamlfmt_path: Option<&PathBuf>,
) -> String {
    let mut root = serde_yaml::Mapping::new();

    for (key, value) in map {
        insert_nested(&mut root, key, value);
    }

    let yaml_value = serde_yaml::Value::Mapping(root);

    // TODO: FIXME: Proper error handling
    let s = serde_yaml::to_string(&yaml_value).unwrap();

    if skip_format {
        if verbose {
            println!("Skipping formatting ...",);
        }
        s
    } else {
        format(&s, verbose, yamlfmt_path)
    }
}

// TODO: Return a result
fn flatten_yaml(value: &serde_yaml::Value, prefix: String, map: &mut IndexMap<String, String>) {
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

// TODO: Return a result
pub fn parse(content: &str) -> IndexMap<String, String> {
    let value: serde_yaml::Value = serde_yaml::from_str(content).expect("failed to parse yaml");
    let mut map = IndexMap::new();

    flatten_yaml(&value, String::new(), &mut map);

    map
}

pub fn format(input: &str, verbose: bool, yamlfmt_path: Option<&PathBuf>) -> String {
    println!("NA FUNCAO FORMAT BRRRR");

    let yaml_bin: &str = match yamlfmt_path {
        Some(path) => path.to_str().unwrap(),
        None => "yamlfmt",
    };

    #[allow(clippy::unnecessary_unwrap)]
    if yamlfmt_path.is_none() {
        // In this case, yamlfmt should be in the PATH
        // panic if it is not
        if which::which("yamlfmt").is_err() {
            panic!(
                "yamlfmt is not installed or not in PATH. Please install yamlfmt from https://github.com/google/yamlfmt"
            );
        }

        if verbose {
            println!("yamlfmt in PATH ...");
            println!("Formatting with yamlfmt ...",);
        }
    } else {
        // Check if the path is valid
        let path = yamlfmt_path.unwrap();

        // The path should exist, be a file, and should be executable

        if !path.exists() {
            panic!("Provided yamlfmt path does not exist: {:?}", path);
        }

        if !path.is_file() {
            panic!("Provided yamlfmt path is not a file: {:?}", path);
        }

        // Check if it is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path).expect("Failed to read file metadata");
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 == 0 {
                panic!("Provided yamlfmt path is not executable: {:?}", path);
            }
        }

        if verbose {
            println!("Using yamlfmt from {:?}", path);
        }
    }

    let tmp_file = std::env::temp_dir().join(format!("props2yml_{}.yml", std::process::id()));

    // Write input to temporary file
    fs::write(&tmp_file, input).expect("Failed to write temporary file");

    // Run yamlfmt on the file
    let status = Command::new(yaml_bin)
        .arg(&tmp_file)
        .status()
        .expect("Failed to execute yamlfmt");

    // Clean up on error
    if !status.success() {
        let _ = fs::remove_file(&tmp_file);
        panic!("yamlfmt failed to format the file");
    }

    // Read the formatted content
    let formatted = fs::read_to_string(&tmp_file).expect("Failed to read formatted file");

    // Delete the temporary file
    fs::remove_file(&tmp_file).expect("Failed to delete temporary file");

    formatted
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
        let mut map = IndexMap::new();
        map.insert("key".to_string(), "value".to_string());
        let output = write(&map, true, false, None);
        assert!(output.contains("key:"));
        assert!(output.contains("value"));
        assert!(output.contains("key: value"),);
    }

    #[test]
    fn test_write_nested() {
        let mut map = IndexMap::new();
        map.insert("database.host".to_string(), "localhost".to_string());
        map.insert("database.port".to_string(), "5432".to_string());

        let output = write(&map, true, false, None);
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
