use crate::errors::Prop2YamlError;
use crate::{properties, yaml};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Properties,
    Yaml,
}

impl Format {
    pub fn from_str(s: &str) -> Result<Self, Prop2YamlError> {
        match s.to_lowercase().as_str() {
            "properties" | "props" | "prop" => Ok(Format::Properties),
            "yaml" | "yml" => Ok(Format::Yaml),
            _ => Err(Prop2YamlError::UnsupportedFormat(s.to_string())),
        }
    }

    pub fn from_path(path: &str) -> Result<Self, Prop2YamlError> {
        if path.ends_with(".properties") {
            Ok(Format::Properties)
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            Ok(Format::Yaml)
        } else {
            Err(Prop2YamlError::UnsupportedFormat(
                "Cannot determine format from file extension".to_string(),
            ))
        }
    }
}

// TODO: This should return a Result
pub fn convert(
    input: &str,
    from: Format,
    to: Format,
    skip_format: bool,
    verbose: bool,
    yamlfmt_path: Option<&PathBuf>,
) -> String {
    let map = match from {
        Format::Properties => properties::parse(input),
        Format::Yaml => yaml::parse(input),
    };

    match to {
        Format::Properties => properties::write(&map),
        Format::Yaml => yaml::write(&map, skip_format, verbose, yamlfmt_path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_str() {
        assert_eq!(Format::from_str("properties").unwrap(), Format::Properties);
        assert_eq!(Format::from_str("yaml").unwrap(), Format::Yaml);
        assert_eq!(Format::from_str("yml").unwrap(), Format::Yaml);
        assert!(Format::from_str("json").is_err());
    }

    #[test]
    fn test_format_from_path() {
        assert_eq!(
            Format::from_path("test.properties").unwrap(),
            Format::Properties
        );
        assert_eq!(Format::from_path("test.yaml").unwrap(), Format::Yaml);
        assert_eq!(Format::from_path("test.yml").unwrap(), Format::Yaml);
    }

    #[test]
    fn test_convert_props_to_yaml() {
        let input = "
        key=value
        foo=bar
        ";
        let output = convert(input, Format::Properties, Format::Yaml, true, false, None);
        assert!(output.contains("key:"));
        assert!(output.contains("value"));
    }

    #[test]
    fn test_convert_yaml_to_props() {
        let input = "
        key=value
        foo=bar
        ";
        let output = convert(input, Format::Yaml, Format::Properties, true, false, None);
        assert!(output.contains("key=value"));
        assert!(output.contains("foo=bar"));
    }
}
