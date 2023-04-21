//! Parsing utilities.

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use crate::query::*;
use anyhow::{anyhow, Result};

/// Parse parameters given as strings.
pub fn parse_parameter_values<'a, T>(
    parameters: &'a [Parameter],
    param_values: &'a HashMap<T, T>,
) -> Result<Vec<VariableParam<'a>>>
where
    T: Borrow<str> + Eq + Hash,
{
    let mut values = Vec::new();
    for param in parameters {
        let value = param_values
            .get(&param.name)
            .ok_or(anyhow!("no value provided for parameter `{}`", param.name))?;
        let value = parse_value(&param.parameter_type, value.borrow())?;
        values.push(VariableParam {
            name: &param.name,
            value,
        });
    }
    Ok(values)
}

fn parse_value<'a>(typ: &ParameterType, value: &'a str) -> Result<ValueParam<'a>> {
    match typ {
        ParameterType::TypeBoolean => Ok(ValueParam::DataBoolean(Some(
            value.to_ascii_lowercase() == "true",
        ))),
        ParameterType::TypeDecimal => value
            .parse()
            .map(Option::Some)
            .map(ValueParam::DataDecimal)
            .map_err(Into::into),
        ParameterType::TypeInteger => value
            .parse()
            .map(Option::Some)
            .map(ValueParam::DataInteger)
            .map_err(Into::into),
        // TODO parse the timestamp here, as early as possible.
        ParameterType::TypeTimestamp => Ok(ValueParam::DataTimestamp(Some(value))),
        ParameterType::TypeString => Ok(ValueParam::DataString(Some(value))),
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
            ValueParam::DataBoolean(Some(true))
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeBoolean, "TRUE")?,
            ValueParam::DataBoolean(Some(true))
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeBoolean, "false")?,
            ValueParam::DataBoolean(Some(false))
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeBoolean, "FALSE")?,
            ValueParam::DataBoolean(Some(false))
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeBoolean, "something")?,
            ValueParam::DataBoolean(Some(false))
        ));
        Ok(())
    }

    #[test]
    fn test_parse_value_integer() -> Result<()> {
        assert!(matches!(
            parse_value(&ParameterType::TypeInteger, "123")?,
            ValueParam::DataInteger(Some(123))
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeInteger, "0")?,
            ValueParam::DataInteger(Some(0))
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeInteger, "-123")?,
            ValueParam::DataInteger(Some(-123))
        ));

        assert!(parse_value(&ParameterType::TypeInteger, "something").is_err());
        assert!(parse_value(&ParameterType::TypeInteger, "true").is_err());
        assert!(parse_value(&ParameterType::TypeInteger, "12.3").is_err());
        Ok(())
    }

    #[test]
    fn test_parse_value_decimal() -> Result<()> {
        assert!(matches!(
            parse_value(&ParameterType::TypeDecimal, "123.4")?,
            ValueParam::DataDecimal(Some(x)) if x == 123.4
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeDecimal, "0")?,
            ValueParam::DataDecimal(Some(x)) if x == 0.0
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeDecimal, "-123.4")?,
            ValueParam::DataDecimal(Some(x)) if x == -123.4
        ));
        assert!(matches!(
            parse_value(&ParameterType::TypeDecimal, "123")?,
            ValueParam::DataDecimal(Some(x)) if x == 123.0
        ));

        assert!(parse_value(&ParameterType::TypeDecimal, "something").is_err());
        assert!(parse_value(&ParameterType::TypeDecimal, "true").is_err());
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
                    value: ValueResult::DataBoolean(Some(true))
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
                        value: ValueResult::DataBoolean(Some(true))
                    },
                    VariableResult {
                        name: "you".to_string(),
                        value: ValueResult::DataBoolean(Some(true))
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
                        value: ValueResult::DataBoolean(Some(true))
                    },
                    VariableResult {
                        name: "world".to_string(),
                        value: ValueResult::DataBoolean(Some(true))
                    }
                ]
            )
        );
    }
}
