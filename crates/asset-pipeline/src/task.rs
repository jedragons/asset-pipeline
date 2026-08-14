use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use anyhow::Context as _;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub struct Task {
    pub action: Arc<dyn Fn(&[PathBuf], &[PathBuf]) -> anyhow::Result<()> + Send + Sync>,
    pub inputs: Vec<PathBuf>,
    pub outputs: Vec<PathBuf>,
}

impl Task {
    pub fn new(
        action: impl Fn(&[PathBuf], &[PathBuf]) -> anyhow::Result<()> + Send + Sync + 'static,
        inputs: Vec<PathBuf>,
        outputs: Vec<PathBuf>,
    ) -> Self {
        Self {
            inputs,
            outputs,
            action: Arc::new(action),
        }
    }

    pub fn execute(&self) -> anyhow::Result<()> {
        let modified = |vec: &[PathBuf]| {
            vec.iter()
                .map(|p| {
                    Ok(p.metadata()
                        .with_context(|| format!("Getting metadata for {p:?}"))?
                        .modified()?)
                })
                .collect::<Result<Vec<SystemTime>, anyhow::Error>>()
        };

        let input_times = modified(&self.inputs).unwrap_or_default();
        let output_times = modified(&self.outputs).unwrap_or_default();

        let newest_input = input_times
            .into_iter()
            .max()
            .unwrap_or_else(|| SystemTime::now());
        let oldest_output = output_times
            .into_iter()
            .min()
            .unwrap_or_else(|| SystemTime::UNIX_EPOCH);

        if newest_input > oldest_output {
            (self.action)(&self.inputs, &self.outputs)?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct TaskStore {
    pub tasks: HashMap<PathBuf, Vec<Arc<Task>>>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    pub fn add(
        &mut self,
        action: impl Fn(&[PathBuf], &[PathBuf]) -> anyhow::Result<()> + Send + Sync + 'static,
        inputs: &[PathBuf],
        outputs: &[PathBuf],
    ) -> anyhow::Result<()> {
        let mut inputs = inputs
            .iter()
            .map(|i| i.canonicalize().unwrap_or(i.clone()))
            .collect::<Vec<_>>();

        if inputs.is_empty() {
            inputs.push(PathBuf::new());
        }

        let outputs = outputs
            .iter()
            .map(|i| i.canonicalize().unwrap_or(i.clone()))
            .collect::<Vec<_>>();

        let task = Arc::new(Task::new(action, inputs.clone(), outputs));

        for input in inputs {
            self.tasks
                .entry(input)
                .or_insert_with(|| Vec::new())
                .push(task.clone());
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn execute(&self, input: &Path) {
        let Some(tasks) = self.tasks.get(input) else {
            return;
        };

        for task in tasks {
            if let Err(e) = task.execute() {
                println!(
                    "ERROR: An error occured while generating {input:?}\nCould not execute task: {e:?}\n\n"
                );
            }
        }
    }

    pub fn execute_all(&self) {
        let mut executed = HashSet::<*const Task>::new();

        let mut tasks = Vec::new();
        for (path, entry) in &self.tasks {
            for task in entry {
                let addr = &**task as *const Task;
                if executed.contains(&addr) {
                    continue;
                }
                executed.insert(addr);

                tasks.push((path, task));
            }
        }

        tasks.into_par_iter().for_each(|(path, task)| {
            if let Err(e) = task.execute() {
                println!(
                    "ERROR: An error occured while generating {path:?}\nCould not execute task: {e:?}\n\n"
                );
            }
        });
    }
}
