//! Machine-readable pin of the `srtla_send` command-line surface.
//!
//! This test walks the `clap::Command` structurally (via
//! `srtla_send::cli::cli_command()`, i.e. `clap::CommandFactory`) rather than
//! parsing argv vectors, so a RENAMED, REORDERED, REMOVED, or RE-DEFAULTED
//! argument fails here even when some other argv spelling still happens to
//! parse.
//!
//! It exists to lock the CeraUI device-integration contract
//! (`AGENTS.md` → PARITY CONTRACT):
//!
//! ```text
//! srtla_send <SRT_LISTEN_PORT> <SRTLA_HOST> <SRTLA_PORT> <BIND_IPS_FILE> [OPTIONS]
//! ```
//!
//! Changing anything asserted below is a deliberate, versioned CLI change —
//! never an incidental refactor or an upstream-merge side effect.

use clap::Command;
use srtla_send::cli::{DEFAULT_STATS_FILE_INTERVAL_MS, cli_command};

/// Positional arguments in declaration/index order — this IS the contract.
const POSITIONAL_CONTRACT: [&str; 4] = [
    "local_srt_port",
    "receiver_host",
    "receiver_port",
    "ips_file",
];

/// `(arg id, rendered long flag, expected default rendering)`.
///
/// `None` means the argument has no default value at all (an `Option<T>` that
/// is simply absent). Boolean switches render their `SetTrue`/`SetFalse`
/// default as `"false"`.
const FLAG_CONTRACT: [(&str, &str, Option<&str>); 17] = [
    ("print_version", "version", Some("false")),
    ("capabilities_json", "capabilities-json", Some("false")),
    ("bind_map", "bind-map", None),
    ("verbose", "verbose", Some("false")),
    ("dry_run", "dry-run", Some("false")),
    ("stats_file", "stats-file", None),
    ("stats_file_interval", "stats-file-interval", Some("1000")),
    ("control_socket", "control-socket", None),
    ("mode", "mode", Some("enhanced")),
    ("no_quality", "no-quality", Some("false")),
    ("exploration", "exploration", Some("false")),
    ("rtt_delta_ms", "rtt-delta-ms", Some("30")),
    ("earned_ack_window", "earned-ack-window", Some("false")),
    ("stall_deselect", "stall-deselect", Some("false")),
    ("stall_min_in_flight", "stall-min-in-flight", Some("32")),
    ("stall_ack_stale_ms", "stall-ack-stale-ms", Some("3000")),
    ("stall_reprobe_ms", "stall-reprobe-ms", Some("1000")),
];

fn command() -> Command {
    let mut cmd = cli_command();
    // Positional indices and action-implied defaults are only materialized once
    // clap has built the command; without this the introspection below sees
    // `None` for every index.
    cmd.build();
    cmd
}

fn defaults_of(cmd: &Command, id: &str) -> Option<Vec<String>> {
    let arg = cmd
        .get_arguments()
        .find(|a| a.get_id().as_str() == id)
        .unwrap_or_else(|| panic!("argument `{id}` is missing from the CLI definition"));
    let values: Vec<String> = arg
        .get_default_values()
        .iter()
        .map(|v| v.to_string_lossy().into_owned())
        .collect();
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

#[test]
fn binary_name_is_srtla_send() {
    // The device integration spawns the binary by this exact name.
    assert_eq!(command().get_name(), "srtla_send");
}

#[test]
fn positional_arguments_keep_their_exact_index_order() {
    let cmd = command();

    let mut positionals: Vec<(usize, String)> = cmd
        .get_arguments()
        .filter(|a| a.is_positional())
        .map(|a| {
            (
                a.get_index()
                    .unwrap_or_else(|| panic!("positional `{}` has no index", a.get_id())),
                a.get_id().to_string(),
            )
        })
        .collect();
    positionals.sort_by_key(|(idx, _)| *idx);

    let ordered: Vec<&str> = positionals.iter().map(|(_, id)| id.as_str()).collect();
    assert_eq!(
        ordered,
        POSITIONAL_CONTRACT.to_vec(),
        "positional order is a load-bearing parity contract: srtla_send <SRT_LISTEN_PORT> \
         <SRTLA_HOST> <SRTLA_PORT> <BIND_IPS_FILE>"
    );

    // Indices must be exactly 1..=4 with no gaps, so "same relative order but
    // an extra positional wedged in" also fails.
    let indices: Vec<usize> = positionals.iter().map(|(idx, _)| *idx).collect();
    assert_eq!(
        indices,
        vec![1, 2, 3, 4],
        "positional indices must be 1..=4"
    );
}

#[test]
fn all_four_positionals_are_required_unless_version_is_requested() {
    let cmd = command();
    for id in POSITIONAL_CONTRACT {
        let arg = cmd
            .get_arguments()
            .find(|a| a.get_id().as_str() == id)
            .unwrap_or_else(|| panic!("positional `{id}` is missing"));
        assert!(
            arg.is_positional(),
            "`{id}` must remain a positional, not a flag"
        );
        assert!(
            arg.get_default_values().is_empty(),
            "`{id}` must not acquire a default that would mask a missing argument"
        );
    }

    let err = command()
        .try_get_matches_from(["srtla_send"])
        .expect_err("omitting the positionals must be an error");
    assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);

    command()
        .try_get_matches_from(["srtla_send", "--version"])
        .expect("--version short-circuits the required positionals");

    command()
        .try_get_matches_from(["srtla_send", "--capabilities-json"])
        .expect("--capabilities-json is a pre-spawn probe: it cannot demand a configuration");
}

#[test]
fn bind_map_is_a_fully_optional_additive_flag() {
    // The four positionals plus an IP-only file must keep working untouched;
    // --bind-map may only ever add to that invocation, never alter it.
    let cmd = command();
    let arg = cmd
        .get_arguments()
        .find(|a| a.get_id().as_str() == "bind_map")
        .expect("--bind-map must exist");
    assert!(!arg.is_required_set(), "--bind-map must never be required");
    assert!(
        arg.get_default_values().is_empty(),
        "--bind-map must have no default: absent means legacy behavior"
    );

    let without = command()
        .try_get_matches_from(["srtla_send", "5000", "127.0.0.1", "5001", "/tmp/ips"])
        .expect("the legacy four-positional invocation must still parse");
    assert_eq!(without.get_one::<String>("bind_map"), None);

    let with = command()
        .try_get_matches_from([
            "srtla_send",
            "5000",
            "127.0.0.1",
            "5001",
            "/tmp/ips",
            "--bind-map",
            "/tmp/srtla_bind_map.json",
        ])
        .expect("--bind-map is accepted alongside the positionals");
    assert_eq!(
        with.get_one::<String>("bind_map").map(String::as_str),
        Some("/tmp/srtla_bind_map.json")
    );
}

#[test]
fn every_documented_flag_exists_with_its_documented_long_name() {
    let cmd = command();
    for (id, long, _) in FLAG_CONTRACT {
        let arg = cmd
            .get_arguments()
            .find(|a| a.get_id().as_str() == id)
            .unwrap_or_else(|| panic!("argument `{id}` is missing from the CLI definition"));
        assert_eq!(
            arg.get_long(),
            Some(long),
            "`{id}` must render as `--{long}`"
        );
        assert!(
            !arg.is_positional(),
            "`{id}` must stay an option/flag, not a positional"
        );
    }
}

#[test]
fn every_documented_flag_keeps_its_documented_default() {
    let cmd = command();
    for (id, long, expected) in FLAG_CONTRACT {
        let actual = defaults_of(&cmd, id);
        match expected {
            Some(expected) => assert_eq!(
                actual.as_deref(),
                Some(&[expected.to_string()][..]),
                "`--{long}` default changed"
            ),
            None => assert_eq!(actual, None, "`--{long}` must have no default value"),
        }
    }
}

#[test]
fn version_flag_keeps_its_short_alias() {
    let cmd = command();
    let arg = cmd
        .get_arguments()
        .find(|a| a.get_id().as_str() == "print_version")
        .expect("print_version argument must exist");
    assert_eq!(arg.get_short(), Some('v'), "`-v` is operator-visible");
    assert_eq!(arg.get_long(), Some("version"));
}

#[test]
fn stats_file_interval_default_matches_the_named_constant() {
    assert_eq!(DEFAULT_STATS_FILE_INTERVAL_MS, 1000);
    assert_eq!(
        defaults_of(&command(), "stats_file_interval"),
        Some(vec![DEFAULT_STATS_FILE_INTERVAL_MS.to_string()])
    );
}

#[test]
fn mode_accepts_exactly_the_four_scheduling_modes() {
    let cmd = command();
    let arg = cmd
        .get_arguments()
        .find(|a| a.get_id().as_str() == "mode")
        .expect("--mode must exist");
    let values: Vec<String> = arg
        .get_possible_values()
        .iter()
        .map(|v| v.get_name().to_string())
        .collect();
    assert_eq!(
        values,
        vec!["classic", "enhanced", "rtt-threshold", "edpf"],
        "the scheduling-mode value set is part of the CLI surface"
    );
}

#[test]
fn the_cli_surface_has_no_undeclared_arguments() {
    // A NEW argument must be added to one of the two contract tables above,
    // which forces a deliberate review of the parity contract.
    let cmd = command();
    let known: Vec<&str> = POSITIONAL_CONTRACT
        .iter()
        .copied()
        .chain(FLAG_CONTRACT.iter().map(|(id, _, _)| *id))
        .chain(["help"])
        .collect();

    let unknown: Vec<String> = cmd
        .get_arguments()
        .map(|a| a.get_id().to_string())
        .filter(|id| !known.contains(&id.as_str()))
        .collect();
    assert!(
        unknown.is_empty(),
        "undeclared CLI arguments found: {unknown:?} — add them to the contract tables in \
         tests/cli_surface.rs deliberately"
    );
}

#[test]
fn usage_string_pins_the_positional_contract() {
    let usage = command().render_usage().to_string();
    assert!(
        usage.contains("srtla_send [OPTIONS] SRT_LISTEN_PORT SRTLA_HOST SRTLA_PORT BIND_IPS_FILE"),
        "override_usage must keep the documented shape; got: {usage}"
    );
}

#[test]
fn main_has_no_duplicate_module_tree() {
    let main_source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"));
    const ALLOWED_TEST_MODULES: &[&str] = &[];
    let duplicate_modules: Vec<&str> = main_source
        .lines()
        .filter_map(|line| line.strip_prefix("mod "))
        .filter(|declaration| {
            let module_name = declaration
                .strip_suffix(';')
                .map(str::trim)
                .unwrap_or_default();
            !ALLOWED_TEST_MODULES.contains(&module_name)
        })
        .collect();

    assert!(
        duplicate_modules.is_empty(),
        "src/main.rs must not declare private modules; found: {duplicate_modules:?}"
    );
}
