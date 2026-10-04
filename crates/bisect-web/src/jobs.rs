use crate::{
    data::{self, Paths},
    model::{Experiment, Job, StateRun},
};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub struct Laboratory {
    pub paths: Paths,
    pub jobs: Mutex<BTreeMap<String, Job>>,
    pub worker: Mutex<()>,
    pub geometry_lock: Mutex<()>,
    pub token: String,
}

impl Laboratory {
    pub fn new(paths: Paths) -> Result<Arc<Self>> {
        let mut jobs = BTreeMap::new();
        for entry in std::fs::read_dir(paths.store.join("jobs"))?.flatten() {
            let file = entry.path().join("experiment.json");
            if let Ok(bytes) = std::fs::read(&file) {
                if let Ok(mut job) = serde_json::from_slice::<Job>(&bytes) {
                    if ["queued", "running"].contains(&job.status.as_str()) {
                        job.status = "interrupted".into();
                        for state in &mut job.states {
                            if ["queued", "running", "preparing"].contains(&state.status.as_str()) {
                                state.status = "interrupted".into();
                            }
                        }
                        data::write_json(&file, &job)?;
                    }
                    jobs.insert(job.id.clone(), job);
                }
            }
        }
        // Cross-site requests cannot read this token; Host and Origin are checked too.
        let token = format!("{:032x}", rand::random::<u128>());
        Ok(Arc::new(Self {
            paths,
            jobs: Mutex::new(jobs),
            worker: Mutex::new(()),
            geometry_lock: Mutex::new(()),
            token,
        }))
    }
    pub fn persist(&self, job: &Job) -> Result<()> {
        data::write_json(
            &self
                .paths
                .store
                .join("jobs")
                .join(&job.id)
                .join("experiment.json"),
            job,
        )
    }
    fn update(&self, id: &str, f: impl FnOnce(&mut Job)) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(id) {
            f(job);
            if let Err(error) = self.persist(job) {
                eprintln!("Cannot persist experiment {id}: {error}");
            }
        }
    }
    pub fn get(&self, id: &str) -> Result<Job> {
        self.jobs
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .context("Experiment not found.")
    }
    pub fn list(&self) -> Value {
        let jobs = self.jobs.lock().unwrap();
        json!(jobs.values().rev().map(|job|json!({"id":job.id,"name":job.config.name,"status":job.status,"mode":job.config.mode,"year":job.config.year,"structure":job.config.structure,"weights":job.config.weights,"search":job.config.search,"state_count":job.states.len(),"completed":job.states.iter().filter(|s|s.status=="completed").count(),"created_unix":job.created_unix})).collect::<Vec<_>>())
    }
    pub fn submit(self: &Arc<Self>, config: Experiment) -> Result<Job> {
        config.validate(&self.paths.codes())?;
        ensure!(
            self.paths.engine.is_file(),
            "Rust engine binary is missing. Build bisect-cli or provide --engine."
        );
        for code in &config.states {
            ensure!(self.paths.available(code,&config.year),"{code} {} has no ready adjacency or local tract/population inputs. Check the data catalog.",config.year);
            if config.districts.is_none() {
                self.paths
                    .district_count(code, &config.year, &config.chamber)?;
            }
        }
        let active = self
            .jobs
            .lock()
            .unwrap()
            .values()
            .filter(|job| ["queued", "running"].contains(&job.status.as_str()))
            .count();
        ensure!(
            active < 8,
            "The laboratory queue is full. Wait for or cancel an experiment."
        );
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let id = format!("lab_{}_{:016x}", now.as_nanos(), rand::random::<u64>());
        let job = Job {
            id: id.clone(),
            states: config
                .states
                .iter()
                .map(|code| StateRun {
                    code: code.clone(),
                    status: "queued".into(),
                    error: None,
                    elapsed_seconds: 0.0,
                    command: vec![],
                    metrics: None,
                })
                .collect(),
            config,
            status: "queued".into(),
            created_unix: now.as_secs(),
            logs: vec![],
        };
        std::fs::create_dir_all(self.paths.store.join("jobs").join(&id))?;
        self.persist(&job)?;
        self.jobs.lock().unwrap().insert(id.clone(), job.clone());
        let app = self.clone();
        std::thread::spawn(move || {
            let _guard = app.worker.lock().unwrap();
            if let Err(error) = app.run(&id) {
                app.update(&id, |j| {
                    j.status = "failed".into();
                    j.logs.push(format!("Laboratory error: {error:#}"));
                });
            }
        });
        Ok(job)
    }
    pub fn cancel(&self, id: &str) -> Result<()> {
        let job = self.get(id)?;
        ensure!(
            ["queued", "running"].contains(&job.status.as_str()),
            "This experiment is already finished."
        );
        self.update(id, |job| job.status = "cancelled".into());
        Ok(())
    }
    fn cancelled(&self, id: &str) -> bool {
        self.jobs
            .lock()
            .unwrap()
            .get(id)
            .is_none_or(|job| job.status == "cancelled")
    }
    fn log(&self, id: &str, line: String) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(id) {
            job.logs.push(line.chars().take(4000).collect());
            if job.logs.len() > 600 {
                job.logs.drain(..100);
            }
        }
    }
    fn run(self: &Arc<Self>, id: &str) -> Result<()> {
        if self.cancelled(id) {
            self.finish_cancelled(id);
            return Ok(());
        }
        self.update(id, |job| job.status = "running".into());
        let initial = self.get(id)?;
        for (index, state) in initial.states.iter().enumerate() {
            if self.cancelled(id) {
                break;
            }
            let started = Instant::now();
            self.update(id, |job| job.states[index].status = "preparing".into());
            self.log(id, format!("{}: preparing Census tract graph", state.code));
            let result = self.run_state(id, index, &state.code, &initial);
            let cancelled = self.cancelled(id);
            self.update(id, |job| {
                let state = &mut job.states[index];
                state.elapsed_seconds = started.elapsed().as_secs_f64();
                match result {
                    Ok(metrics) => {
                        state.metrics = Some(metrics);
                        state.status = "completed".into();
                    }
                    Err(error) => {
                        state.status = if cancelled { "cancelled" } else { "failed" }.into();
                        state.error = Some(format!("{error:#}"));
                    }
                }
            });
        }
        if self.cancelled(id) {
            self.finish_cancelled(id);
        } else {
            self.update(id, |job| {
                let successful = job
                    .states
                    .iter()
                    .filter(|state| state.status == "completed")
                    .count();
                job.status = if successful == job.states.len() {
                    "completed"
                } else if successful > 0 {
                    "partial"
                } else {
                    "failed"
                }
                .into();
            });
        }
        Ok(())
    }
    fn finish_cancelled(&self, id: &str) {
        self.update(id, |job| {
            for state in &mut job.states {
                if ["queued", "preparing", "running"].contains(&state.status.as_str()) {
                    state.status = "cancelled".into();
                }
            }
        });
    }
    fn run_state(self: &Arc<Self>, id: &str, index: usize, code: &str, job: &Job) -> Result<Value> {
        let preparation_started = Instant::now();
        let graph_path = {
            let _guard = self.geometry_lock.lock().unwrap();
            data::prepare(&self.paths, code, &job.config.year)?
        };
        let preparation_seconds = preparation_started.elapsed().as_secs_f64();
        ensure!(
            !self.cancelled(id),
            "Experiment cancelled after preparation."
        );
        let k = match job.config.districts {
            Some(k) => k,
            None => self
                .paths
                .district_count(code, &job.config.year, &job.config.chamber)?,
        };
        let label = format!("{id}_{}", code.to_lowercase());
        let output = self.paths.store.join("jobs").join(id).join("engine");
        let args = job.config.cli_args(
            code,
            k,
            &graph_path.to_string_lossy(),
            &output.to_string_lossy(),
            &label,
        );
        self.update(id, |job| {
            job.states[index].status = "running".into();
            job.states[index].command =
                std::iter::once(self.paths.engine.to_string_lossy().to_string())
                    .chain(args.iter().cloned())
                    .collect();
        });
        let engine_sha256 = crate::catalog::digest(&std::fs::read(&self.paths.engine)?);
        let mut child = Command::new(&self.paths.engine)
            .current_dir(&self.paths.root)
            .args(&args)
            .env("RAYON_NUM_THREADS", "4")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Could not start the Rust engine.")?;
        let mut readers = Vec::new();
        for stream in [
            Box::new(child.stdout.take().unwrap()) as Box<dyn std::io::Read + Send>,
            Box::new(child.stderr.take().unwrap()),
        ] {
            let app = self.clone();
            let id = id.to_owned();
            let code = code.to_owned();
            readers.push(std::thread::spawn(move || {
                for line in BufReader::new(stream).lines() {
                    match line {
                        Ok(line) => app.log(&id, format!("[{code}] {line}")),
                        Err(_) => break,
                    }
                }
            }));
        }
        let start = Instant::now();
        let status = loop {
            if self.cancelled(id) || start.elapsed().as_secs() > job.config.timeout_seconds {
                let _ = child.kill();
                let _ = child.wait();
                for reader in readers {
                    let _ = reader.join();
                }
                anyhow::bail!(if self.cancelled(id) {
                    "Experiment cancelled."
                } else {
                    "State engine time limit exceeded."
                });
            }
            if let Some(status) = child.try_wait()? {
                break status;
            }
            std::thread::sleep(Duration::from_millis(150));
        };
        for reader in readers {
            let _ = reader.join();
        }
        if !status.success() {
            let prefix = format!("[{code}] FAILED {code}: ");
            let record = self.get(id)?;
            let detail = record
                .logs
                .iter()
                .rev()
                .find_map(|line| line.strip_prefix(&prefix));
            anyhow::bail!(
                "Engine exited with {status}: {}",
                detail.unwrap_or("Inspect the run log for details.")
            );
        }
        let engine_seconds = start.elapsed().as_secs_f64();
        ensure!(
            engine_sha256 == crate::catalog::digest(&std::fs::read(&self.paths.engine)?),
            "Engine executable changed during execution."
        );
        let _guard = self.geometry_lock.lock().unwrap();
        let mut metrics = data::collect_result(&self.paths, job, code, k, &graph_path)?;
        metrics["preparation_seconds"] = json!(preparation_seconds);
        metrics["engine_seconds"] = json!(engine_seconds);
        metrics["engine_sha256"] = json!(engine_sha256);
        Ok(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_jobs_survive_restart_without_claiming_success() {
        let temp = tempfile::tempdir().unwrap();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let paths = Paths::new(root, temp.path().to_path_buf(), "missing".into()).unwrap();
        let dir = paths.store.join("jobs/test");
        std::fs::create_dir_all(&dir).unwrap();
        let job = Job {
            id: "test".into(),
            config: crate::model::fixture(),
            status: "running".into(),
            created_unix: 0,
            states: vec![StateRun {
                code: "RI".into(),
                status: "preparing".into(),
                error: None,
                elapsed_seconds: 0.0,
                command: vec![],
                metrics: None,
            }],
            logs: vec![],
        };
        data::write_json(&dir.join("experiment.json"), &job).unwrap();
        let lab = Laboratory::new(paths).unwrap();
        assert_eq!(lab.get("test").unwrap().status, "interrupted");
        assert_eq!(lab.get("test").unwrap().states[0].status, "interrupted");
    }
}
