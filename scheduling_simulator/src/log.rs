use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use crate::task::dag::{Node, DAG};
use chrono::Utc;
use log::info;
use petgraph::Graph;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
struct DAGLog {
    dag_id: usize,
    release_times: Vec<i32>,
    finish_times: Vec<i32>,
    response_times: Vec<i32>,
    best_response_time: i32,
    average_response_time: f32,
    worst_response_time: i32,
}

impl DAGLog {
    pub fn new(dag_id: usize) -> Self {
        Self {
            dag_id,
            release_times: Default::default(),
            finish_times: Default::default(),
            response_times: Default::default(),
            best_response_time: Default::default(),
            average_response_time: Default::default(),
            worst_response_time: Default::default(),
        }
    }

    pub fn calc_response_times(&mut self) {
        // Unequal lengths indicate that the DAG was not completed within the hyper_period, and deadline miss occurred.
        if self.release_times.len() != self.finish_times.len() {
            // Mark as a deadline miss by maximizing the response time.
            self.finish_times.push(std::i32::MAX);
        }

        self.response_times = self
            .release_times
            .iter()
            .zip(self.finish_times.iter())
            .map(|(release_time, finish_time)| *finish_time - *release_time)
            .collect();

        self.best_response_time = *self.response_times.iter().min().unwrap();
        self.average_response_time =
            self.response_times.iter().sum::<i32>() as f32 / self.response_times.len() as f32;
        self.worst_response_time = *self.response_times.iter().max().unwrap();
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
struct ProcessorLog {
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
    dag_set_log: Vec<DAGLog>,
    processor_log: ProcessorLog,
}

impl DAGSetSchedulerLog {
    pub fn new(dag_set: &[Graph<Node, i32>], num_cores: usize) -> Self {
        let mut dag_set_log = Vec::with_capacity(dag_set.len());
        for dag in dag_set.iter() {
            dag_set_log.push(DAGLog::new(dag.get_dag_param("dag_id") as usize));
        }

        Self {
            dag_set_log,
            processor_log: ProcessorLog::new(num_cores),
        }
    }

    pub fn write_dag_release_time(&mut self, dag_i: usize, release_time: i32) {
        self.dag_set_log[dag_i].release_times.push(release_time);
    }

    pub fn write_dag_finish_time(&mut self, dag_i: usize, finish_time: i32) {
        self.dag_set_log[dag_i].finish_times.push(finish_time);
    }

    pub fn write_processing_time(&mut self, core_indices: &[usize]) {
        for core_index in core_indices {
            self.processor_log.core_logs[*core_index].total_proc_time += 1;
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

    pub fn dump_to_yaml(&self, dir_path: &str, alg_name: &str) {
        let date = Utc::now().format("%Y-%m-%d-%H-%M-%S-%3f").to_string();
        let file_name = format!("{}-{}-log", date, alg_name);
        if fs::metadata(dir_path).is_err() {
            let _ = fs::create_dir_all(dir_path);
            info!("Directory created: {}", dir_path);
        }
        let file_path = format!("{}/{}.yaml", dir_path, file_name);
        fs::File::create(&file_path).expect("Failed to create file.");

        let yaml = serde_yaml::to_string(&self).expect("Failed to serialize.");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .expect("Failed to open file.");
        file.write_all(yaml.as_bytes())
            .expect("Failed to write to file.");
    }
}
