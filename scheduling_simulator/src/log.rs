use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
};

use crate::{
    processor::core::ProcessResult,
    task::dag::{Node, DAG},
};
use chrono::Utc;
use log::info;
use petgraph::Graph;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGLog {
    dag_id: usize,
    release_times: Vec<i32>,
    finish_times: HashMap<usize, Vec<i32>>, // sink_i -> finish_times
    pub response_times_per_sink: HashMap<usize, Vec<i32>>, // sink_i -> response_times
    best_response_time_per_sink: HashMap<usize, i32>,
    average_response_time_per_sink: HashMap<usize, f32>,
    worst_response_time_per_sink: HashMap<usize, i32>,
}

impl DAGLog {
    pub fn new(dag_id: usize) -> Self {
        Self {
            dag_id,
            release_times: Default::default(),
            finish_times: Default::default(),
            response_times_per_sink: Default::default(),
            best_response_time_per_sink: Default::default(),
            average_response_time_per_sink: Default::default(),
            worst_response_time_per_sink: Default::default(),
        }
    }

    pub fn calc_response_times(&mut self) {
        for (&sink_i, rts) in &self.response_times_per_sink {
            let mut sum_rt = 0;
            let mut min_rt = i32::MAX;
            let mut max_rt = i32::MIN;
            for &rt in rts {
                sum_rt += rt;
                min_rt = min_rt.min(rt);
                max_rt = max_rt.max(rt);
            }

            self.best_response_time_per_sink.insert(sink_i, min_rt);
            self.average_response_time_per_sink
                .insert(sink_i, sum_rt as f32 / rts.len() as f32);
            self.worst_response_time_per_sink.insert(sink_i, max_rt);
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct CoreLog {
    core_id: usize,
    total_proc_time: i32,
    utilization: f32,
}

impl CoreLog {
    fn new(core_id: usize) -> Self {
        Self {
            core_id,
            total_proc_time: Default::default(),
            utilization: Default::default(),
        }
    }

    fn calculate_utilization(&mut self, schedule_length: i32) {
        self.utilization = self.total_proc_time as f32 / schedule_length as f32;
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ProcessorLog {
    num_cores: usize,
    core_logs: Vec<CoreLog>,
    average_utilization: f32,
    variance_utilization: f32,
}

impl ProcessorLog {
    pub fn new(num_cores: usize) -> Self {
        Self {
            num_cores,
            core_logs: (0..num_cores).map(CoreLog::new).collect(),
            average_utilization: Default::default(),
            variance_utilization: Default::default(),
        }
    }

    fn calc_utilization(&mut self, schedule_length: i32) {
        for core_log in self.core_logs.iter_mut() {
            core_log.calculate_utilization(schedule_length);
        }

        self.average_utilization = self
            .core_logs
            .iter()
            .map(|core_log| core_log.utilization)
            .sum::<f32>()
            / self.core_logs.len() as f32;

        self.variance_utilization = self
            .core_logs
            .iter()
            .map(|core_log| (core_log.utilization - self.average_utilization).powi(2))
            .sum::<f32>()
            / self.core_logs.len() as f32;
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DAGSetSchedulerLog {
    pub deadline_missed: bool,
    pub total_utilization: f32,
    pub dag_set_log: Vec<DAGLog>,
    pub processor_log: ProcessorLog,
}

impl DAGSetSchedulerLog {
    pub fn new(dag_set: &[Graph<Node, i32>], num_cores: usize) -> Self {
        let total_utilization = dag_set.iter().map(|dag| dag.get_utilization()).sum::<f32>();
        let mut dag_set_log = Vec::with_capacity(dag_set.len());
        for dag in dag_set.iter() {
            dag_set_log.push(DAGLog::new(dag.get_dag_param("dag_id") as usize));
        }

        Self {
            deadline_missed: false,
            total_utilization,
            dag_set_log,
            processor_log: ProcessorLog::new(num_cores),
        }
    }

    pub fn write_dag_release_time(&mut self, dag_i: usize, release_time: i32) {
        self.dag_set_log[dag_i].release_times.push(release_time);
    }

    pub fn write_dag_finish_time(&mut self, dag_i: usize, sink_i: usize, finish_time: i32) -> i32 {
        let dag_log = &mut self.dag_set_log[dag_i];
        let finish_times = dag_log.finish_times.entry(sink_i).or_default();
        let response_time = finish_time - dag_log.release_times[finish_times.len()];
        dag_log
            .response_times_per_sink
            .entry(sink_i)
            .or_default()
            .push(response_time);
        finish_times.push(finish_time);

        response_time
    }

    pub fn write_processing_time(&mut self, process_result: &[ProcessResult]) {
        let core_indices = process_result
            .iter()
            .enumerate()
            .filter_map(|(index, result)| match result {
                ProcessResult::InProgress => Some(index),
                ProcessResult::Done(node_data) if !node_data.params.contains_key("dummy") => {
                    Some(index)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        for core_i in core_indices {
            self.processor_log.core_logs[core_i].total_proc_time += 1;
        }
    }

    pub fn calc_response_times(&mut self) {
        for dag_log in self.dag_set_log.iter_mut() {
            dag_log.calc_response_times();
        }
    }

    pub fn calculate_utilization(&mut self, schedule_length: i32) {
        self.processor_log.calc_utilization(schedule_length);
    }

    pub fn dump_to_yaml(&self, dir_path: &str, alg_name: &str, verbose: bool) {
        let date = Utc::now().format("%Y-%m-%d-%H-%M-%S-%3f").to_string();
        let file_name = format!("{}-{}-log", date, alg_name);
        if fs::metadata(dir_path).is_err() {
            let _ = fs::create_dir_all(dir_path);
            info!("Directory created: {}", dir_path);
        }
        let file_path = format!("{}/{}.yaml", dir_path, file_name);
        fs::File::create(&file_path).expect("Failed to create file.");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .expect("Failed to open file.");
        let yaml = if verbose {
            serde_yaml::to_string(&self).expect("Failed to serialize.")
        } else {
            serde_yaml::to_string(&HashMap::from([
                ("deadline_missed", self.deadline_missed.to_string()),
                ("total_utilization", self.total_utilization.to_string()),
                ("num_cores", self.processor_log.num_cores.to_string()),
            ]))
            .expect("Failed to serialize.")
        };
        file.write_all(yaml.as_bytes())
            .expect("Failed to write to file.");
    }
}
