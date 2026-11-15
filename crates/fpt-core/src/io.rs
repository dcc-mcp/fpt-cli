use crate::error::{AppError, Result};
use serde_json::Value;
use std::fs;
use std::io::Read;

pub fn read_json_input(input: Option<&str>) -> Result<Option<Value>> {
    input.map_or_else(|| Ok(None), |source| read_from_source(source).map(Some))
}

fn read_from_source(source: &str) -> Result<Value> {
    if source == "@-" {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| {
                AppError::invalid_input(format!("could not read JSON input from stdin: {error}"))
                    .with_operation("read_json_input")
                    .with_input_source("stdin")
                    .with_hint("Provide JSON through stdin or switch to inline JSON / @file input.")
            })?;

        return parse_json(&buffer, "stdin");
    }

    if let Some(path) = source.strip_prefix('@') {
        let content = fs::read_to_string(path).map_err(|error| {
            AppError::invalid_input(format!("could not read JSON input file `{path}`: {error}"))
                .with_operation("read_json_input")
                .with_input_source(path)
                .with_resource(path)
                .with_hint("Check that the file exists, is readable, and contains JSON text.")
        })?;

        return parse_json(&content, path);
    }

    parse_json(source, "inline JSON")
}

fn parse_json(raw: &str, label: &str) -> Result<Value> {
    serde_json::from_str(raw).map_err(|error| {
        AppError::invalid_input(format!(
            "could not parse {label} as JSON; provide valid JSON text, a file reference like `@input.json`, or `@-` for stdin: {error}"
        ))
        .with_operation("parse_json_input")
        .with_input_source(label)
        .with_expected_shape("valid JSON text, a file reference like `@input.json`, or `@-` for stdin")
        .with_hint("Validate the JSON syntax and ensure the payload is complete before retrying.")
    })
}
