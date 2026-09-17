use super::*;

fn profile_environment(value: Option<&str>) -> Result<Vec<(String, String)>> {
    match value {
        None => Ok(Vec::new()),
        Some(value @ ("converged" | "legacy-l1" | "legacy-l2")) => {
            Ok(vec![("SLS_BONDED_PROFILE_OVERRIDE".into(), value.into())])
        }
        Some(_) => bail!("unsupported SLS_BONDED_PROFILE_OVERRIDE"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rollback_environment_is_forwarded_without_replacing_default() {
        // Given the default and the real rollback setting, when building child env,
        // then the default stays unset and rollback is not silently converged.
        assert!(super::profile_environment(None).unwrap().is_empty());
        assert_eq!(
            super::profile_environment(Some("legacy-l2")).unwrap(),
            vec![("SLS_BONDED_PROFILE_OVERRIDE".into(), "legacy-l2".into())]
        );
        assert!(super::profile_environment(Some("typo")).is_err());
    }
}

impl SrtSink {
    pub fn start_sls_listener(&self, ns: &Namespace) -> Result<(NamespaceProcess, SlsCapture)> {
        let (binary, template) = match self {
            Self::Slt => bail!("SLS listener requires SrtSink::Sls"),
            Self::Sls {
                binary,
                conf_template,
            } => (binary, conf_template),
        };
        let directory = tempfile::tempdir().context("SLS run directory")?;
        let conf = render_sls_conf(&std::fs::read_to_string(template)?, directory.path())?;
        let path = directory.path().join("sls.conf");
        std::fs::write(&path, conf)?;
        let process = NamespaceProcess::spawn_launch(
            process_control::Launch {
                namespace: ns.name.clone(),
                binary: binary.to_str().context("SLS binary UTF-8")?.into(),
                args: vec![
                    "-c".into(),
                    path.to_str().context("SLS config UTF-8")?.into(),
                ],
                env: profile_environment(
                    std::env::var("SLS_BONDED_PROFILE_OVERRIDE").ok().as_deref(),
                )?,
            },
            process_control::Teardown::Process,
        )?;
        let output = directory.path().join("player.ts");
        Ok((
            process,
            SlsCapture {
                player: None,
                directory,
                output,
            },
        ))
    }
}

impl SlsCapture {
    pub fn attach_player(&mut self, ns: &Namespace, tool: &Path) -> Result<()> {
        anyhow::ensure!(self.player.is_none(), "SLS player already attached");
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if Self::scrape(ns)?.publishers.contains_key(SLS_STREAM) {
                break;
            }
            anyhow::ensure!(Instant::now() < deadline, "publisher not registered");
            std::thread::sleep(Duration::from_millis(50));
        }
        self.player = Some(NamespaceProcess::spawn_process_only(
            ns,
            "sh",
            &[
                "-c",
                "exec \"$1\" -statsout \"$4\" -statspf:csv -stats 100 \"$2\" file://con > \"$3\"",
                "sls-player",
                tool.to_str().context("SRT tool UTF-8")?,
                "srt://127.0.0.1:4000?mode=caller&streamid=play/live/conformance&latency=200",
                self.output.to_str().context("player output UTF-8")?,
                self.player_stats_path()
                    .to_str()
                    .context("player CSV UTF-8")?,
            ],
        )?);
        loop {
            if std::fs::metadata(&self.output).is_ok_and(|m| m.len() > 0) {
                return Ok(());
            }
            let player = self.player.as_mut().context("SLS player handle")?;
            anyhow::ensure!(
                player.is_alive() && Instant::now() < deadline,
                "SLS player failed to carry: {:?}",
                player.log_snapshot()
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn begin_measurement(&self) -> Result<SlsMeasurement> {
        anyhow::ensure!(
            self.player.is_some(),
            "attach the SLS player before measuring"
        );
        Ok(SlsMeasurement {
            start_bytes: std::fs::metadata(&self.output)?.len(),
            started: Instant::now(),
            output: self.output.clone(),
        })
    }

    pub fn player_stats_path(&self) -> PathBuf {
        self.directory.path().join("player.csv")
    }

    pub fn player_bytes(&self) -> Result<u64> {
        Ok(std::fs::metadata(&self.output)?.len())
    }

    pub fn raw_stats(ns: &Namespace) -> Result<String> {
        let output = ns.exec_checked(
            "curl",
            &[
                "--noproxy",
                "*",
                "--max-time",
                "2",
                "-fsS",
                "-H",
                "Authorization: bpc-conformance-local",
                "http://127.0.0.1:8181/stats",
            ],
        )?;
        let text = String::from_utf8(output.stdout)?;
        parse_sls_stats(&text)?;
        Ok(text)
    }

    pub fn finish_measurement(
        &self,
        endpoints: (&Namespace, &NamespaceProcess),
        measurement: SlsMeasurement,
        offered: (u64, u32),
    ) -> Result<SlsConformanceRecord> {
        let (ns, server) = endpoints;
        let (offered_bytes, device_preset_ms) = offered;
        anyhow::ensure!(
            measurement.output == self.output,
            "measurement belongs to another stack"
        );
        let end_bytes = std::fs::metadata(&self.output)?.len();
        let duration_ms = u32::try_from(measurement.started.elapsed().as_millis())?;
        let (player_bytes, useful_goodput_bps) =
            sls_player_goodput(measurement.start_bytes, end_bytes, duration_ms)?;
        let stats = Self::scrape(ns)?;
        let sls_stats = stats.publishers.get(SLS_STREAM).cloned();
        let mut assertions = SlsAssertions::evaluate(
            sls_stats.as_ref(),
            player_bytes,
            offered_bytes,
            device_preset_ms,
        );
        if server
            .log_snapshot()
            .iter()
            .any(|line| line.contains("failed to read latency"))
        {
            assertions.latency = false;
        }
        let record = SlsConformanceRecord {
            sink: "sls",
            metrics: "none",
            assertions,
            sls_stats,
            offered_bytes,
            player_bytes,
            duration_ms,
            useful_goodput_bps,
            pkt_rcv_belated: None,
            occupancy_pct: None,
            belated_gate: "not_applicable",
            occupancy_gate: "not_applicable",
        };
        std::fs::write(
            self.directory.path().join("conformance.json"),
            serde_json::to_vec_pretty(&record)?,
        )?;
        Ok(record)
    }

    pub fn save_artifacts(&self, directory: &Path) -> Result<()> {
        for name in ["sls.conf", "player.ts", "player.csv", "conformance.json"] {
            let path = self.directory.path().join(name);
            if path.exists() {
                std::fs::copy(path, directory.join(name))?;
            }
        }
        if let Some(player) = &self.player {
            std::fs::write(
                directory.join("player.log"),
                player.log_snapshot().join("\n"),
            )?;
        }
        Ok(())
    }
}
