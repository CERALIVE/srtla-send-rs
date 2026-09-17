use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use network_sim::harness::SrtSink;
use network_sim::metrics::identity::Hash256;
use network_sim::metrics::sls::SlsIdentity;
use serde::Deserialize;

use super::record::binary_hash;

#[derive(Deserialize)]
struct Lock {
    conformance_sinks: Sinks,
}
#[derive(Deserialize)]
struct Sinks {
    sls: LockedSls,
}
#[derive(Deserialize)]
struct LockedSls {
    sha256: Hash256,
    dependencies: Vec<Dependency>,
}
#[derive(Deserialize)]
struct Dependency {
    path: PathBuf,
    sha256: Hash256,
}

pub fn resolve() -> Result<(SrtSink, SlsIdentity)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let template = root.join("tests/bench_support/sls-conformance.conf.tmpl");
    let sink = SrtSink::sls_from_env(template.clone())?;
    let binary = match &sink {
        SrtSink::Sls { binary, .. } => binary,
        SrtSink::Slt => anyhow::bail!("SLS resolver returned a metric sink"),
    };
    let lock: Lock = serde_json::from_slice(&std::fs::read(
        root.join("scripts/bench/receivers.lock.json"),
    )?)?;
    let locked = lock.conformance_sinks.sls;
    ensure!(
        binary_hash(binary)? == locked.sha256,
        "SLS_BIN differs from receivers.lock.json"
    );
    ensure!(
        locked.dependencies.len() == 1,
        "SLS requires one locked libsrt dependency"
    );
    let dependency = &locked.dependencies[0];
    ensure!(
        binary_hash(&dependency.path)? == dependency.sha256,
        "SLS libsrt hash changed"
    );
    let output = std::process::Command::new("ldd").arg(binary).output()?;
    ensure!(output.status.success(), "resolve SLS dynamic dependencies");
    let text = std::str::from_utf8(&output.stdout)?;
    let loaded = text
        .lines()
        .find_map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next()?;
            (name.starts_with("libsrt.so") && words.next()? == "=>")
                .then(|| words.next())
                .flatten()
        })
        .context("SLS has no resolved shared libsrt")?;
    ensure!(
        PathBuf::from(loaded).canonicalize()? == dependency.path.canonicalize()?,
        "SLS loads a different libsrt than its lock"
    );
    let identity = SlsIdentity {
        binary_sha256: locked.sha256,
        template_sha256: binary_hash(&template)?,
        libsrt_sha256: dependency.sha256.clone(),
    };
    Ok((sink, identity))
}
