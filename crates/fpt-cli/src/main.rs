//! `fpt` — CLI entry point for Flow Production Tracking operations.
//!
//! Parses command-line arguments via `clap`, dispatches to the domain layer,
//! and prints structured JSON (or TOON) output to stdout/stderr.

#![allow(clippy::result_large_err)]

mod cli;
mod config;
mod output;
mod runner;
mod self_update;

use clap::Parser;
use cli::Cli;
use fpt_core::OutputFormat;
use serde_json::Value;
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let output_format: OutputFormat = cli.output.into();
    let result = runner::run(cli).await;

    match result {
        Ok(value) => {
            output::print_stdout(&value, output_format);
            let exit_code = logical_failure_exit_code(&value);
            process::exit(exit_code);
        }
        Err(error) => {
            output::print_stderr(&error.envelope(), output_format);
            process::exit(error.exit_code());
        }
    }
}

/// Returns a non-zero exit code when the response signals a logical failure:
///
/// - `{"exception": true, ...}` — RPC-level exception (e.g. entity summarize)
/// - `{"ok": false, ...}` — top-level operation failure
/// - `{"failure_count": N, ...}` where N > 0 — batch operation with failures
///
/// Returns `0` when the response indicates success or is not a recognized failure shape.
fn logical_failure_exit_code(value: &Value) -> i32 {
    let Some(obj) = value.as_object() else {
        return 0;
    };

    // RPC exception: {"exception": true, ...}
    if obj.get("exception").and_then(Value::as_bool) == Some(true) {
        return 1;
    }

    // Top-level ok: false
    if obj.get("ok").and_then(Value::as_bool) == Some(false) {
        return 1;
    }

    // Batch operation with failures: {"failure_count": N, ...} where N > 0
    // Defense-in-depth: batch helpers already set "ok": false when failure_count > 0,
    // but this check ensures correct exit codes even if a future code path omits "ok".
    if obj
        .get("failure_count")
        .and_then(Value::as_u64)
        .is_some_and(|n| n > 0)
    {
        return 1;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn success_response_returns_exit_zero() {
        let value = json!({"ok": true, "data": []});
        assert_eq!(logical_failure_exit_code(&value), 0);
    }

    #[test]
    fn non_object_returns_exit_zero() {
        assert_eq!(logical_failure_exit_code(&json!("text")), 0);
        assert_eq!(logical_failure_exit_code(&json!(42)), 0);
        assert_eq!(logical_failure_exit_code(&json!(null)), 0);
        assert_eq!(logical_failure_exit_code(&json!([1, 2])), 0);
    }

    #[test]
    fn rpc_exception_returns_exit_one() {
        let value = json!({"exception": true, "message": "malformed payload"});
        assert_eq!(logical_failure_exit_code(&value), 1);
    }

    #[test]
    fn exception_false_returns_exit_zero() {
        let value = json!({"exception": false, "data": {}});
        assert_eq!(logical_failure_exit_code(&value), 0);
    }

    #[test]
    fn ok_false_returns_exit_one() {
        let value = json!({"ok": false, "error": "something went wrong"});
        assert_eq!(logical_failure_exit_code(&value), 1);
    }

    #[test]
    fn batch_all_failures_returns_exit_one() {
        let value = json!({
            "ok": false,
            "operation": "entity.batch.find",
            "total": 3,
            "success_count": 0,
            "failure_count": 3,
        });
        assert_eq!(logical_failure_exit_code(&value), 1);
    }

    #[test]
    fn batch_partial_failure_returns_exit_one() {
        let value = json!({
            "ok": false,
            "operation": "entity.batch.create",
            "total": 5,
            "success_count": 3,
            "failure_count": 2,
        });
        assert_eq!(logical_failure_exit_code(&value), 1);
    }

    #[test]
    fn batch_all_success_returns_exit_zero() {
        let value = json!({
            "ok": true,
            "operation": "entity.batch.find",
            "total": 3,
            "success_count": 3,
            "failure_count": 0,
        });
        assert_eq!(logical_failure_exit_code(&value), 0);
    }

    #[test]
    fn failure_count_without_ok_returns_exit_one() {
        // Defense-in-depth: even without "ok" field, failure_count > 0 triggers non-zero exit
        let value = json!({
            "operation": "entity.batch.find",
            "failure_count": 2,
        });
        assert_eq!(logical_failure_exit_code(&value), 1);
    }

    #[test]
    fn empty_object_returns_exit_zero() {
        let value = json!({});
        assert_eq!(logical_failure_exit_code(&value), 0);
    }
}
