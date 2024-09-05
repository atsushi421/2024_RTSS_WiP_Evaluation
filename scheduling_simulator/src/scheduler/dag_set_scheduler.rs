use crate::{
    log::DAGSetSchedulerLog,
    processor::{core::ProcessResult, processor_interface::Processor},
    task::dag::{Node, DAG},
};
use petgraph::graph::Graph;
use std::collections::VecDeque;

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
    fn release_dags(
        &mut self,
        ready_queue: &mut VecDeque<Node>,
        uncompleted_dag_jobs: &mut Vec<Graph<Node, i32>>,
    ) {
        let current_time = self.get_current_time();
        let mut dag_set = self.get_dag_set();

        for dag in dag_set.iter_mut() {
            let job_i = dag.get_dag_param("job_id");
            if current_time == dag.get_dag_param("period") * job_i {
                Self::update_params_when_release(dag, job_i);
                ready_queue.push_back(dag[dag.get_source()].clone());
                uncompleted_dag_jobs.push(dag.clone());
                self.get_log_mut()
                    .write_dag_release_time(dag.get_dag_param("dag_id") as usize, current_time);
                dag.set_param_to_all_nodes("job_id", job_i + 1);
            }
        }

        self.set_dag_set(dag_set);
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
        uncompleted_dags: &mut Vec<Graph<Node, i32>>,
    ) -> Result<Vec<Node>, String> {
        let dag_id = node.get_value("dag_id");
        let owner_dag = uncompleted_dags
            .iter_mut()
            .find(|dag| {
                dag.get_dag_param("dag_id") == dag_id
                    && dag.get_dag_param("job_id") == node.get_value("job_id")
            })
            .unwrap();
        let current_time = self.get_current_time();

        let mut triggered_nodes = Vec::new();
        let suc_nodes = owner_dag.get_suc(node.get_id());
        if suc_nodes.is_empty() {
            let response_time = self
                .get_log_mut()
                .write_dag_finish_time(dag_id as usize, current_time);
            if response_time > node.get_value("relative_deadline") {
                return Err("Deadline missed".to_string());
            }
            uncompleted_dags.retain(|dag| {
                !(dag.get_dag_param("dag_id") == dag_id
                    && dag.get_dag_param("job_id") == node.get_value("job_id"))
            });
        } else {
            for suc in suc_nodes {
                if owner_dag[suc].params.contains_key("pre_done_count") {
                    owner_dag.update_param(
                        suc,
                        "pre_done_count",
                        owner_dag[suc].get_value("pre_done_count") + 1,
                    );
                } else {
                    owner_dag.add_param(suc, "pre_done_count", 1);
                }
                if owner_dag.is_node_ready(suc) {
                    triggered_nodes.push(owner_dag[suc].clone());
                }
            }
        }

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

    fn calculate_log(&mut self, deadline_missed: bool) {
        let current_time = self.get_current_time();
        let log = self.get_log_mut();
        log.calculate_utilization(current_time);
        log.calc_response_times();
        log.deadline_missed = deadline_missed;
    }

    fn schedule(&mut self, preemptive_type: PreemptiveType, duration: i32) -> i32 {
        // Initialize job_id
        let mut dag_set = self.get_dag_set();
        for dag in dag_set.iter_mut() {
            dag.set_param_to_all_nodes("job_id", 0);
        }
        self.set_dag_set(dag_set);

        // Start scheduling
        let mut deadline_missed = false;
        let mut ready_queue = VecDeque::new();
        let mut uncompleted_dag_jobs = Vec::new();

        'outer: while self.get_current_time() < duration {
            // Release DAGs
            self.release_dags(&mut ready_queue, &mut uncompleted_dag_jobs);
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
                    if let Ok(triggered_nodes) =
                        self.node_completion(node_data, &mut uncompleted_dag_jobs)
                    {
                        for triggered_node in triggered_nodes {
                            ready_queue.push_back(triggered_node);
                        }
                    } else {
                        deadline_missed = true;
                        break 'outer;
                    }
                }
            }
            self.sort_ready_queue(&mut ready_queue);
        }

        self.calculate_log(deadline_missed);
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
