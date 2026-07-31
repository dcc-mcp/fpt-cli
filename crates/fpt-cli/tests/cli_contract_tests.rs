use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn capabilities_uses_json_as_default_output() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\":\"fpt\""));
}

#[test]
fn capabilities_still_supports_explicit_toon_output() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "toon"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("name: fpt"));
}

#[test]
fn capabilities_outputs_entity_update_contract() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entity.update\""));
}

#[test]
fn capabilities_lists_user_password_auth_mode() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command.assert().success().stdout(
        predicate::str::contains("\"user_password\"")
            .and(predicate::str::contains("\"FPT_USERNAME\""))
            .and(predicate::str::contains("\"FPT_SESSION_TOKEN\"")),
    );
}

#[test]
fn inspect_command_shows_dry_run_support() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "entity.update", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"supports_dry_run\":true"));
}

#[test]
fn inspect_auth_test_mentions_session_token_mode() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "auth.test", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("session-token"));
}

#[test]
fn inspect_auth_login_mentions_secure_profiles() {
    let mut command = Command::cargo_bin("fpt").expect("fpt binary");
    command.args(["inspect", "command", "auth.login", "--output", "json"]);
    command
        .assert()
        .success()
        .stdout(predicate::str::contains("system_keyring"))
        .stdout(predicate::str::contains(
            "operating system credential store",
        ));
}

#[test]
fn inspect_entity_find_mentions_structured_search() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "entity.find", "--output", "json"]);

    command.assert().success().stdout(
        predicate::str::contains("filter-dsl")
            .and(predicate::str::contains("structured `search` object"))
            .and(predicate::str::contains("additional_filter_presets")),
    );
}

#[test]
fn inspect_entity_find_one_is_registered() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "entity.find-one", "--output", "json"]);

    command.assert().success().stdout(
        predicate::str::contains("first matching record or null")
            .and(predicate::str::contains("same as entity.find")),
    );
}

#[test]
fn inspect_entity_summarize_is_registered() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "entity.summarize", "--output", "json"]);

    command.assert().success().stdout(
        predicate::str::contains("ShotGrid summary operators")
            .and(predicate::str::contains("summary_fields"))
            .and(predicate::str::contains("summarize")),
    );
}

#[test]
fn entity_update_dry_run_outputs_rest_request_plan() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "entity",
        "update",
        "Task",
        "42",
        "--input",
        "{\"data\":{\"type\":\"Task\",\"id\":42}}",
        "--dry-run",
        "--output",
        "json",
    ]);

    command.assert().success().stdout(
        predicate::str::contains("\"dry_run\":true")
            .and(predicate::str::contains("/api/v1.1/entity/tasks/42")),
    );
}

#[test]
fn work_schedule_read_requires_input_json() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["work-schedule", "read", "--output", "json"]);

    command.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

#[test]
fn delete_without_yes_is_blocked() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["entity", "delete", "Task", "42", "--output", "json"]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("POLICY_BLOCKED"));
}

// --- #49: capabilities version must match the CLI binary version ---

#[test]
fn capabilities_version_matches_cli_version() {
    let expected_version = env!("CARGO_PKG_VERSION");
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "\"version\":\"{expected_version}\""
        )));
}

// --- #48: config clear must not panic ---

#[test]
fn config_clear_all_does_not_panic() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["config", "clear", "--all", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"config.clear\""));
}

#[test]
fn config_clear_fields_does_not_panic() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "config",
        "clear",
        "--fields",
        "site,auth-mode",
        "--output",
        "json",
    ]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"config.clear\""));
}

#[test]
fn config_clear_no_args_returns_error() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["config", "clear", "--output", "json"]);

    command.assert().failure().stderr(
        predicate::str::contains("INVALID_INPUT")
            .and(predicate::str::contains("--all"))
            .and(predicate::str::contains("--fields")),
    );
}

#[test]
fn config_clear_invalid_field_returns_error() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "config",
        "clear",
        "--fields",
        "nonexistent",
        "--output",
        "json",
    ]);

    command.assert().failure().stderr(
        predicate::str::contains("INVALID_INPUT")
            .and(predicate::str::contains("unknown field name")),
    );
}

// --- config get / config path tests ---

#[test]
fn config_get_outputs_config_command_tag() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["config", "get", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"config.get\""));
}

#[test]
fn config_path_outputs_config_path_command_tag() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["config", "path", "--output", "json"]);

    command.assert().success().stdout(
        predicate::str::contains("\"command\":\"config.path\"")
            .and(predicate::str::contains("\"path\":")),
    );
}

// --- config set tests ---

#[test]
fn config_set_no_args_returns_error() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["config", "set", "--output", "json"]);

    command.assert().failure().stderr(
        predicate::str::contains("INVALID_INPUT").and(predicate::str::contains("config set")),
    );
}

#[test]
fn config_set_with_site_succeeds() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "config",
        "set",
        "--site",
        "https://test.shotgrid.autodesk.com",
        "--output",
        "json",
    ]);

    command.assert().success().stdout(
        predicate::str::contains("\"command\":\"config.set\"")
            .and(predicate::str::contains("test.shotgrid.autodesk.com")),
    );
}

// --- entity create dry-run ---

#[test]
fn entity_create_dry_run_outputs_rest_request_plan() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "entity",
        "create",
        "Shot",
        "--input",
        "{\"data\":{\"type\":\"Shot\",\"attributes\":{\"code\":\"test\"}}}",
        "--dry-run",
        "--output",
        "json",
    ]);

    command.assert().success().stdout(
        predicate::str::contains("\"dry_run\":true")
            .and(predicate::str::contains("/api/v1.1/entity/shots")),
    );
}

// --- entity delete dry-run ---

#[test]
fn entity_delete_dry_run_outputs_rest_request_plan() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "entity",
        "delete",
        "Task",
        "42",
        "--dry-run",
        "--output",
        "json",
    ]);

    command.assert().success().stdout(
        predicate::str::contains("\"dry_run\":true")
            .and(predicate::str::contains("/api/v1.1/entity/tasks/42"))
            .and(predicate::str::contains("DELETE")),
    );
}

// --- entity revive dry-run ---

#[test]
fn entity_revive_dry_run_outputs_rpc_request_plan() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "entity",
        "revive",
        "Shot",
        "860",
        "--dry-run",
        "--output",
        "json",
    ]);

    command.assert().success().stdout(
        predicate::str::contains("\"dry_run\":true").and(predicate::str::contains("revive")),
    );
}

// --- inspect registered command validation ---

#[test]
fn inspect_hierarchy_search_is_registered() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "hierarchy.search", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("hierarchy"));
}

#[test]
fn inspect_entity_text_search_is_registered() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "inspect",
        "command",
        "entity.text-search",
        "--output",
        "json",
    ]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("text"));
}

#[test]
fn inspect_unknown_command_returns_error() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args([
        "inspect",
        "command",
        "nonexistent.command",
        "--output",
        "json",
    ]);

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("UNSUPPORTED_CAPABILITY"));
}

#[test]
fn capabilities_outputs_hierarchy_expand_contract() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"hierarchy.expand\""));
}

#[test]
fn capabilities_outputs_schedule_work_day_rules_contract() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schedule.work-day-rules\""));
}

#[test]
fn capabilities_outputs_license_get_contract() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"license.get\""));
}

#[test]
fn capabilities_outputs_preferences_custom_entity_contract() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["capabilities", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"preferences.custom-entity\""));
}

#[test]
fn inspect_hierarchy_expand_command() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "hierarchy.expand", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"hierarchy.expand\""));
}

#[test]
fn inspect_license_get_command() {
    let mut command = Command::cargo_bin("fpt").expect("binary exists");
    command.args(["inspect", "command", "license.get", "--output", "json"]);

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("\"license.get\""));
}
