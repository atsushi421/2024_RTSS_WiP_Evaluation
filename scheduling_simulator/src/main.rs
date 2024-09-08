use clap::Parser;
use scheduling_simulator::{
    processor::{homogeneous::HomogeneousProcessor, processor_interface::Processor},
    scheduler::{
        dag_set_scheduler::{DAGSetSchedulerBase, PreemptiveType},
        fixed_priority_scheduler::FixedPriorityScheduler,
        proposed_edf_scheduler::GlobalEDFScheduler,
    },
    task::{dag::DAG, dag_creator::create_dag_set_from_dir},
};

#[derive(Parser)]
struct ArgParser {
    /// Path to DAGSet directory.
    #[clap(short = 'd', long = "dag_dir_path", default_value = "../autoware_dags")]
    dag_dir_path: String,
    /// Number of processing cores.
    #[clap(short = 'c', long = "num_cores", required = true)]
    num_cores: usize,
    /// Simulation duration.
    #[clap(short = 's', long = "sim_duration", required = true)]
    sim_duration: i32,
    /// Path to output directory.
    #[clap(short = 'o', long = "output_dir_path", default_value = "../outputs")]
    output_dir_path: String,
}

fn main() {
    let arg: ArgParser = ArgParser::parse();
    let dag_set = create_dag_set_from_dir(&arg.dag_dir_path);

    // The reason why the all algorithms are executed in the same main function is
    // to use the same DAGSet that is randomly generated.

    // Proposed EDF
    let processor = HomogeneousProcessor::new(arg.num_cores);
    let mut scheduler = GlobalEDFScheduler::new(&dag_set.clone(), &processor);
    scheduler.schedule(
        PreemptiveType::Preemptive {
            key: "ref_absolute_deadline".to_string(),
        },
        arg.sim_duration,
    );
    scheduler.dump_log(&arg.output_dir_path, "proposed_edf", false);

    // RM
    let processor = HomogeneousProcessor::new(arg.num_cores);
    let mut rm_dag_set = dag_set.clone();
    for dag in rm_dag_set.iter_mut() {
        let dag_period = dag.get_dag_param("period");
        for node in dag.node_weights_mut() {
            node.params.insert("priority".to_string(), dag_period);
        }
    }
    let mut scheduler = FixedPriorityScheduler::new(&rm_dag_set, &processor);
    scheduler.schedule(
        PreemptiveType::Preemptive {
            key: "priority".to_string(),
        },
        arg.sim_duration,
    );
    scheduler.dump_log(&arg.output_dir_path, "rm", false);

    // Greedy
    let processor = HomogeneousProcessor::new(arg.num_cores);
    let mut greedy_dag_set = dag_set.clone();
    let uniform_priority = 0;
    for dag in greedy_dag_set.iter_mut() {
        for node in dag.node_weights_mut() {
            node.params.insert("priority".to_string(), uniform_priority);
        }
    }
    let mut scheduler = FixedPriorityScheduler::new(&greedy_dag_set, &processor);
    scheduler.schedule(PreemptiveType::NonPreemptive, arg.sim_duration);
    scheduler.dump_log(&arg.output_dir_path, "greedy", false);
}
