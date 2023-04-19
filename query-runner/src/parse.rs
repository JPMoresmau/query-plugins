//! Parsing utilities.

use std::collections::HashMap;

use crate::query::*;
use anyhow::{anyhow, Result};

/// Parse parameters given as strings.
pub(crate) fn parse_parameter_values<'a>(
    parameters: &'a [Parameter],
    param_values: &'a HashMap<&str, &str>,
) -> Result<Vec<VariableParam<'a>>> {
    let mut values = Vec::new();
    for param in parameters {
        let value = param_values
            .get(param.name.as_str())
            .ok_or(anyhow!("no value provided for parameter `{}`", param.name))?;
        let value = parse_value(&param.parameter_type, value)?;
        values.push(VariableParam {
            name: &param.name,
            value,
        });
    }
    Ok(values)
}

fn parse_value<'a>(typ: &ParameterType, value: &'a str) -> Result<ValueParam<'a>> {
    match typ {
        ParameterType::TypeBoolean => Ok(ValueParam::DataBoolean(
            value.to_ascii_lowercase() == "true",
        )),
        ParameterType::TypeDecimal => value
            .parse()
            .map(ValueParam::DataDecimal)
            .map_err(Into::into),
        ParameterType::TypeInteger => value
            .parse()
            .map(ValueParam::DataInteger)
            .map_err(Into::into),
        // TODO parse the timestamp here, as early as possible.
        ParameterType::TypeTimestamp => Ok(ValueParam::DataTimestamp(value)),
        ParameterType::TypeString => Ok(ValueParam::DataString(value)),
    }
}

/// Replace {{param}} by positional index.
pub(crate) fn positional(
    prefix: &str,
    offset: usize,
    query: &str,
    params: &[VariableResult],
) -> String {
    let mut replaced = query.to_string();
    for (ix, param) in params.iter().enumerate() {
        // TODO support spaces between curlies and parameter name?
        replaced = replaced.replace(
            &format!("{{{{{}}}}}", param.name),
            &format!("{prefix}{}", ix + offset),
        );
    }
    replaced
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_value_bool() -> Result<()> {
        assert!(matches!(
            parse_value(&ParameterType::TypeBoolean, "true")?,
            ValueParam::DataBoolean(true)
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeBoolean, "TRUE")?,
            ValueParam::DataBoolean(true)
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeBoolean, "false")?,
            ValueParam::DataBoolean(false)
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeBoolean, "FALSE")?,
            ValueParam::DataBoolean(false)
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeBoolean, "something")?,
            ValueParam::DataBoolean(false)
        ));
        Ok(())
    }

    #[test]
    fn test_positional() {
        assert_eq!("hello", positional("$", 1, "hello", &[]));
        assert_eq!(
            "hello $1",
            positional(
                "$",
                1,
                "hello {{world}}",
                &[VariableResult {
                    name: "world".to_string(),
                    value: ValueResult::DataBoolean(true)
                }]
            )
        );
        assert_eq!(
            "hello $1, how are $2",
            positional(
                "$",
                1,
                "hello {{world}}, how are {{you}}",
                &[
                    VariableResult {
                        name: "world".to_string(),
                        value: ValueResult::DataBoolean(true)
                    },
                    VariableResult {
                        name: "you".to_string(),
                        value: ValueResult::DataBoolean(true)
                    }
                ]
            )
        );
        assert_eq!(
            "hello $2, how are $1",
            positional(
                "$",
                1,
                "hello {{world}}, how are {{you}}",
                &[
                    VariableResult {
                        name: "you".to_string(),
                        value: ValueResult::DataBoolean(true)
                    },
                    VariableResult {
                        name: "world".to_string(),
                        value: ValueResult::DataBoolean(true)
                    }
                ]
            )
        );
    }
}
