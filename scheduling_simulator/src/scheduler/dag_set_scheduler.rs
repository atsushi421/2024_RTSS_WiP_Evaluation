use crate::{
    log::DAGSetSchedulerLog,
    processor::{core::ProcessResult, processor_interface::Processor},
    task::dag::{Node, DAG},
};
use petgraph::graph::Graph;
use std::collections::VecDeque;

#[derive(Clone, Default, PartialEq)]
enum DAGState {
    #[default]
    Waiting,
    Ready,
}

/// This is associated with a single DAG.
#[derive(Clone, Default)]
pub struct DAGStateManager {
    dag_state: DAGState,
    release_count: i32,
}

impl DAGStateManager {
    fn get_release_count(&self) -> i32 {
        self.release_count
    }

    fn get_dag_state(&self) -> DAGState {
        self.dag_state.clone()
    }

    fn complete_execution(&mut self) {
        self.dag_state = DAGState::Waiting;
    }

    fn release(&mut self) {
        self.release_count += 1;
        self.dag_state = DAGState::Ready;
    }
}

pub enum PreemptiveType {
    NonPreemptive,
    Preemptive { key: String },
}

pub trait DAGSetSchedulerBase<T: Processor + Clone> {
    // getter, setter
    fn get_dag_set(&self) -> Vec<Graph<Node, i32>>;
    fn set_dag_set(&mut self, dag_set: Vec<Graph<Node, i32>>);
    fn get_processor(&self) -> &T;
    fn get_processor_mut(&mut self) -> &mut T;
    fn get_log(&self) -> &DAGSetSchedulerLog;
    fn get_log_mut(&mut self) -> &mut DAGSetSchedulerLog;
    fn get_current_time(&self) -> i32;
    fn get_current_time_mut(&mut self) -> &mut i32;

    // method definition
    fn new(dag_set: &[Graph<Node, i32>], processor: &T) -> Self;
    fn sort_ready_queue(&self, ready_queue: &mut VecDeque<Node>);
    fn update_params_when_release(dag: &mut Graph<Node, i32>, job_id: i32);

    // method implementation
    fn release_dags(&mut self, managers: &mut [DAGStateManager]) -> Vec<Node> {
        let mut ready_nodes = Vec::new();
        let current_time = self.get_current_time();
        let mut dag_set = self.get_dag_set();

        for dag in dag_set.iter_mut() {
            let dag_i = dag.get_dag_param("dag_id") as usize;
            if (managers[dag_i].get_dag_state() == DAGState::Waiting)
                && (current_time == dag.get_dag_period() * managers[dag_i].get_release_count())
            {
                managers[dag_i].release();
                Self::update_params_when_release(dag, managers[dag_i].get_release_count());
                ready_nodes.push(dag[dag.get_source()[0]].clone());
                self.get_log_mut()
                    .write_dag_release_time(dag_i, current_time);
            }
        }
        self.set_dag_set(dag_set);

        ready_nodes
    }

    fn process_unit_time(&mut self) -> Vec<ProcessResult> {
        let current_time = self.get_current_time_mut();
        *current_time += 1;
        let process_result = self.get_processor_mut().process();
        self.get_log_mut().write_processing_time(&process_result);

        process_result
    }

    fn node_completion(
        &mut self,
        node: &Node,
        managers: &mut [DAGStateManager],
    ) -> Result<Vec<Node>, String> {
        let mut dag_set = self.get_dag_set();
        let dag_id = node.get_value("dag_id") as usize;
        let dag = &mut dag_set[dag_id];
        let current_time = self.get_current_time();

        let mut triggered_nodes = Vec::new();
        let suc_nodes = dag.get_suc(node.get_id());
        if suc_nodes.is_empty() {
            let response_time = self
                .get_log_mut()
                .write_dag_finish_time(dag_id, current_time);
            if response_time > dag.get_dag_param("relative_deadline") {
                return Err("Deadline missed".to_string());
            }

            dag.set_param_to_all_nodes("pre_done_count", 0);
            managers[dag_id].complete_execution();
        } else {
            for suc in suc_nodes {
                if dag[suc].params.contains_key("pre_done_count") {
                    dag.update_param(
                        suc,
                        "pre_done_count",
                        dag[suc].get_value("pre_done_count") + 1,
                    );
                } else {
                    dag.add_param(suc, "pre_done_count", 1);
                }
                if dag.is_node_ready(suc) {
                    triggered_nodes.push(dag[suc].clone());
                }
            }
        }

        self.set_dag_set(dag_set);

        Ok(triggered_nodes)
    }

    fn can_preempt(
        &self,
        preemptive_type: &PreemptiveType,
        ready_head_node: &Node,
    ) -> Option<usize> {
        if let PreemptiveType::Preemptive {
            key: preemptive_key,
        } = &preemptive_type
        {
            let (max_value, core_i) = self
                .get_processor()
                .get_max_and_index(preemptive_key)
                .unwrap();

            if max_value > ready_head_node.get_value(preemptive_key) {
                return Some(core_i);
            }
        }

        None
    }

    fn calculate_log(&mut self) {
        let current_time = self.get_current_time();
        let log = self.get_log_mut();
        log.calculate_utilization(current_time);
        log.calc_response_times();
    }

    fn schedule(&mut self, preemptive_type: PreemptiveType, duration: i32) -> i32 {
        // Start scheduling
        let mut managers = vec![DAGStateManager::default(); self.get_dag_set().len()];
        let mut ready_queue = VecDeque::new();

        'outer: while self.get_current_time() < duration {
            // Release DAGs
            for ready_node in self.release_dags(&mut managers) {
                ready_queue.push_back(ready_node);
            }
            self.sort_ready_queue(&mut ready_queue);

            // Allocate nodes as long as there are idle cores, and attempt to preempt when all cores are busy.
            while !ready_queue.is_empty() {
                if let Some(idle_core_i) = self.get_processor().get_idle_core_i() {
                    self.get_processor_mut()
                        .allocate(idle_core_i, &ready_queue.pop_front().unwrap());
                } else if let Some(core_i) =
                    self.can_preempt(&preemptive_type, ready_queue.front().unwrap())
                {
                    // Preempt the node with the lowest priority
                    let processor = self.get_processor_mut();
                    ready_queue.push_back(processor.preempt(core_i));
                    processor.allocate(core_i, &ready_queue.pop_front().unwrap());
                    self.sort_ready_queue(&mut ready_queue);
                } else {
                    break; // No core is idle and can not preempt. Exit the loop.
                }
            }

            // Process unit time
            let process_result = self.process_unit_time();

            // Post-process on completion of node execution
            for result in process_result.iter() {
                if let ProcessResult::Done(node_data) = result {
                    if let Ok(triggered_nodes) = self.node_completion(node_data, &mut managers) {
                        for triggered_node in triggered_nodes {
                            ready_queue.push_back(triggered_node);
                        }
                    } else {
                        break 'outer; // Deadline missed
                    }
                }
            }
            self.sort_ready_queue(&mut ready_queue);
        }

        self.calculate_log();
        self.get_current_time()
    }

    fn dump_log(&mut self, dir_path: &str, alg_name: &str) {
        self.get_log_mut().dump_to_yaml(dir_path, alg_name);
    }
}

#[macro_export]
macro_rules! dag_set_scheduler_common {
    { $t:ty } => {
        fn get_dag_set(&self) -> Vec<Graph<Node, i32>>{
            self.dag_set.clone()
        }
        fn set_dag_set(&mut self, dag_set: Vec<Graph<Node, i32>>){
            self.dag_set = dag_set;
        }
        fn get_processor(&self) -> &$t{
            &self.processor
        }
        fn get_processor_mut(&mut self) -> &mut $t{
            &mut self.processor
        }
        fn get_log(&self) -> &DAGSetSchedulerLog{
            &self.log
        }
        fn get_log_mut(&mut self) -> &mut DAGSetSchedulerLog{
            &mut self.log
        }
        fn get_current_time(&self) -> i32{
            self.current_time
        }
        fn get_current_time_mut(&mut self) -> &mut i32{
            &mut self.current_time
        }

        fn new(dag_set: &[Graph<Node, i32>], processor: &$t) -> Self {
            Self {
                dag_set: dag_set.to_vec(),
                processor: processor.clone(),
                log: DAGSetSchedulerLog::new(dag_set, processor.get_num_cores()),
                current_time: 0,
            }
        }
    }
}
