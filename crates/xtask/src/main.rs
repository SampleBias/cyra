use cyra_core::artifact::{canonical_json, sha256};
use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("xtask: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    std::env::set_current_dir(&root)?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("audit") => println!("{}", serde_json::to_string_pretty(&audit()?)?),
        Some("schemas") => {
            fs::create_dir_all("schemas")?;
            for name in [
                "request",
                "result",
                "scientific-query",
                "artifact-reference",
            ] {
                let schema = cyra_core::service::schema(name).ok_or("missing schema")?;
                fs::write(
                    format!("schemas/{name}.v1.json"),
                    format!("{}\n", serde_json::to_string_pretty(&schema)?),
                )?;
            }
            for name in [
                "import-mapping",
                "dataset-manifest",
                "observation-request",
                "measurement-record",
            ] {
                let schema = cyra_data::schema(name).ok_or("missing data schema")?;
                fs::write(
                    format!("schemas/{name}.v1.json"),
                    format!("{}\n", serde_json::to_string_pretty(&schema)?),
                )?;
            }
        }
        Some("evidence") => collect_evidence(
            &root,
            args.get(1).ok_or("usage: xtask evidence <new-directory>")?,
            "00",
        )?,
        Some("evidence-sprint-01") => collect_evidence(
            &root,
            args.get(1)
                .ok_or("usage: xtask evidence-sprint-01 <new-directory>")?,
            "01",
        )?,
        _ => {
            return Err(
                "usage: xtask audit | schemas | evidence <new-directory> | evidence-sprint-01 <new-directory>"
                    .into(),
            );
        }
    }
    Ok(())
}

fn host_triple() -> Result<String> {
    let output = Command::new("rustc").arg("-vV").output()?;
    if !output.status.success() {
        return Err("rustc -vV failed".into());
    }
    let stdout = String::from_utf8(output.stdout)?;
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or_else(|| "rustc host triple missing".into())
}

fn audit() -> Result<Value> {
    // Review the host resolve only. Target-only crates, such as the Haiku
    // build script `cc` and wasm `links` crates pulled by time-zone support,
    // are not compiled into this build. A host dependency on a banned package
    // still fails.
    let host = host_triple()?;
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--offline",
            "--locked",
            "--format-version",
            "1",
            "--filter-platform",
            &host,
        ])
        .output()?;
    if !output.status.success() {
        return Err("cargo metadata failed".into());
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)?;
    let mut packages = Vec::new();
    let permitted = [
        "MIT/Apache-2.0", // version_check: both license files reviewed.
        "MIT OR Apache-2.0",
        "MIT OR Apache-2.0 OR Zlib",
        "Apache-2.0 OR MIT",
        "MIT",
        "Apache-2.0",
        "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
        "MIT OR Apache-2.0 OR LGPL-2.1-or-later",
        "Unlicense OR MIT",
        "(MIT OR Apache-2.0) AND Unicode-3.0",
        "Zlib OR Apache-2.0 OR MIT",
        // arrow-array 60.0.0: Apache-2.0 LICENSE.txt plus the MIT chronoutil notice.
        "Apache-2.0 AND MIT",
        // tiny-keccak 2.0.2: CC0-1.0 text reviewed. Public-domain dedication.
        "CC0-1.0",
        // zerocopy 0.8.63: BSD-2-Clause, Apache-2.0, and MIT files reviewed.
        "BSD-2-Clause OR Apache-2.0 OR MIT",
    ];
    let banned = [
        "cc",
        "cmake",
        "pyo3",
        "pyo3-ffi",
        "napi",
        "openssl-sys",
        "libz-sys",
        "sha2-asm",
        "torch-sys",
    ];
    for package in metadata["packages"]
        .as_array()
        .ok_or("missing package metadata")?
    {
        let name = package["name"].as_str().ok_or("package name missing")?;
        if package["source"].is_null() {
            continue;
        }
        let license = package["license"]
            .as_str()
            .ok_or("dependency missing license")?;
        if !permitted.contains(&license) {
            return Err(format!("unreviewed dependency license: {name}: {license}").into());
        }
        if banned.contains(&name) || !package["links"].is_null() {
            return Err(format!("native/interpreter dependency requires review: {name}").into());
        }
        packages.push(json!({"name":name,"version":package["version"],"license":license,"source":package["source"],"build_script":package["targets"].as_array().is_some_and(|t| t.iter().any(|v| v["kind"].as_array().is_some_and(|a| a.iter().any(|k| k=="custom-build"))))}));
    }
    packages.sort_by_key(|p| format!("{}:{}", p["name"], p["version"]));
    Ok(
        json!({"status":"passed","scope":format!("locked dependency license/native-build policy for host {host}; target-only crates excluded; not a vulnerability audit"),"host":host,"packages":packages}),
    )
}

fn private_write(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn source_inventory(path: &Path, root: &Path, entries: &mut Vec<Value>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        let name = entry.file_name();
        if [
            "target", ".build", ".git", ".codex", ".agents", ".aws", "evidence",
        ]
        .iter()
        .any(|skip| name == *skip)
        {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            source_inventory(&p, root, entries)?;
        } else if kind.is_file() {
            let bytes = fs::read(&p)?;
            entries.push(json!({"path":p.strip_prefix(root)?.to_string_lossy(),"sha256":sha256(&bytes),"size_bytes":bytes.len()}));
        }
    }
    Ok(())
}

fn collect_evidence(root: &Path, out: &str, sprint: &str) -> Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    let out = PathBuf::from(out);
    if out.exists() {
        return Err("evidence directory already exists; choose a new run ID".into());
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::DirBuilder::new().mode(0o700).create(&out)?;
    // Tests run with the package directory as their working directory, so a
    // relative evidence path would be written under that crate instead of here.
    let out = out.canonicalize()?;
    let mut records = Vec::new();
    let mut all_passed = true;
    let commands: Vec<(&str, Vec<&str>)> = vec![
        ("rustc", vec!["-vV"]),
        ("cargo", vec!["fmt", "--all", "--check"]),
        (
            "cargo",
            vec![
                "clippy",
                "--workspace",
                "--all-targets",
                "--offline",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
        ),
        (
            "cargo",
            vec!["test", "--workspace", "--offline", "--locked"],
        ),
        (
            "cargo",
            vec!["build", "--workspace", "--release", "--offline", "--locked"],
        ),
        (
            "cargo",
            vec!["tree", "--offline", "--locked", "-e", "features"],
        ),
    ];
    for (index, (program, args)) in commands.iter().enumerate() {
        let start = Instant::now();
        let mut command = Command::new(program);
        command.args(args);
        if sprint == "01" && *program == "cargo" && args.contains(&"test") {
            command.env("CYRA_SPRINT01_EVIDENCE", &out);
        }
        let output = command.output()?;
        let stdout_name = format!("check-{index:02}.stdout");
        let stderr_name = format!("check-{index:02}.stderr");
        private_write(&out.join(&stdout_name), &output.stdout)?;
        private_write(&out.join(&stderr_name), &output.stderr)?;
        all_passed &= output.status.success();
        records.push(json!({"program":program,"arguments":args,"exit_code":output.status.code(),"duration_ms":start.elapsed().as_millis(),"stdout":{"path":stdout_name,"sha256":sha256(&output.stdout)},"stderr":{"path":stderr_name,"sha256":sha256(&output.stderr)}}));
    }
    let audit = audit()?;
    let audit_bytes = canonical_json(&audit)?;
    private_write(&out.join("dependency-audit.json"), &audit_bytes)?;
    let binary = root.join("target/release/vmcell");
    let start = Instant::now();
    let output = Command::new(&binary)
        .env_clear()
        .env("PATH", "/nonexistent")
        .args(["--offline", "--format", "json", "version"])
        .output()?;
    all_passed &= output.status.success();
    let value: Value = serde_json::from_slice(&output.stdout)?;
    all_passed &= value["result"]["project"] == "Cyra";
    private_write(&out.join("interpreter-free-version.json"), &output.stdout)?;
    records.push(json!({"program":"target/release/vmcell","arguments":["--offline","--format","json","version"],"environment":"cleared; PATH=/nonexistent; no credentials","exit_code":output.status.code(),"duration_ms":start.elapsed().as_millis(),"stdout_sha256":sha256(&output.stdout)}));
    let performance = benchmark(&binary)?;
    all_passed &= performance["budget_met"].as_bool().unwrap_or(false);
    let performance_bytes = canonical_json(&performance)?;
    private_write(&out.join("performance.json"), &performance_bytes)?;
    let mut inventory = Vec::new();
    source_inventory(root, root, &mut inventory)?;
    inventory.sort_by_key(|v| v["path"].as_str().unwrap_or_default().to_owned());
    let inventory_bytes = canonical_json(&json!(inventory))?;
    private_write(&out.join("source-inventory.json"), &inventory_bytes)?;
    let mut reports = Vec::new();
    if sprint == "01" {
        for name in [
            "round-trip.json",
            "chunk-access.json",
            "interrupted-import.json",
            "split-leakage.json",
            "license-eligibility.json",
        ] {
            let bytes = match fs::read(out.join(name)) {
                Ok(bytes) => bytes,
                Err(_) => {
                    all_passed = false;
                    reports.push(json!({"path": name, "missing": true}));
                    continue;
                }
            };
            reports
                .push(json!({"path": name, "sha256": sha256(&bytes), "size_bytes": bytes.len()}));
        }
        let register = fs::read("docs/acquisition/sprint-01-register.json")?;
        let register_value: Value = serde_json::from_slice(&register)?;
        if register_value["approved"] != json!([]) {
            all_passed = false;
        }
        reports.push(json!({"path":"docs/acquisition/sprint-01-register.json","sha256":sha256(&register),"approved":[]}));
    }
    let manifest = json!({
        "schema_version":"cyra.sprint-evidence.v1","sprint":sprint,"kind":"engineering_verification",
        "status":if all_passed {"passed"} else {"failed"},
        "executed_by": if sprint == "00" { "Codex implementation agent" } else { "implementation agent; independent review pending" },
        "independent_review":"pending","physiological_validation":"not_performed",
        "sprint_reports": reports,
        "platform":{"os":std::env::consts::OS,"arch":std::env::consts::ARCH},
        "source_inventory_sha256":sha256(&inventory_bytes),"binary_sha256":sha256(&fs::read(&binary)?),
        "lockfile_sha256":sha256(&fs::read("Cargo.lock")?),"checks":records,
        "dependency_audit_sha256":sha256(&audit_bytes),"performance_sha256":sha256(&performance_bytes),
        "recorded_at_unix_seconds":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        "network_namespace":fs::read_link("/proc/self/ns/net").ok().map(|p|p.to_string_lossy().into_owned()),
        "network_interfaces":fs::read_to_string("/proc/net/dev").ok(),
        "network_routes":fs::read_to_string("/proc/net/route").ok(),
        "network_isolation":"inspect recorded namespace/interfaces/routes and outer invocation; not inferred from --offline alone"
    });
    private_write(&out.join("manifest.json"), &canonical_json(&manifest)?)?;
    println!(
        "Evidence: {} ({})",
        out.display(),
        if all_passed { "passed" } else { "failed" }
    );
    if !all_passed {
        return Err(io::Error::other("one or more evidence checks failed").into());
    }
    Ok(())
}

fn benchmark(binary: &Path) -> Result<Value> {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    let mut samples = Vec::new();
    for i in 0..33 {
        let start = Instant::now();
        let output = Command::new(binary)
            .env_clear()
            .env("PATH", "/nonexistent")
            .args(["--offline", "--format", "json", "capabilities"])
            .output()?;
        if !output.status.success() {
            return Err("benchmark command failed".into());
        }
        if i >= 3 {
            samples.push(start.elapsed().as_micros());
        }
    }
    samples.sort_unstable();
    let p50 = samples[14];
    let p95 = samples[28];
    let mut child = Command::new(binary)
        .env_clear()
        .env("PATH", "/nonexistent")
        .arg("agent")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut input = child.stdin.take().ok_or("agent stdin unavailable")?;
    writeln!(
        input,
        "{}",
        json!({"schema_version":"vmcell.protocol.v1","id":"bench","operation":"version","arguments":{}})
    )?;
    input.flush()?;
    let mut response = String::new();
    BufReader::new(child.stdout.take().ok_or("agent stdout unavailable")?)
        .read_line(&mut response)?;
    let value: Value = serde_json::from_str(&response)?;
    if value["status"] != "success" {
        return Err("agent benchmark request failed".into());
    }
    let memory = fs::read_to_string(format!("/proc/{}/status", child.id()))?;
    let rss = memory
        .lines()
        .find(|l| l.starts_with("VmHWM:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|n| n.parse::<u64>().ok())
        .ok_or("VmHWM unavailable")?;
    drop(input);
    if !child.wait()?.success() {
        return Err("agent benchmark session failed".into());
    }
    Ok(
        json!({"workload":"release capabilities process startup; resident agent after one version request",
        "samples":30,"warmups":3,"latency_microseconds":{"p50":p50,"p95":p95,"sorted_samples":samples},
        "agent_peak_rss_kib_at_observation":rss,"budgets":{"p95_microseconds":100000,"agent_rss_kib":65536},
        "budget_met":p95<=100000 && rss<=65536,"scientific_latency":"not_applicable_no_simulator"}),
    )
}
