use clap::{Parser, ValueEnum};
use scheduling_simulator::{
    processor::{homogeneous::HomogeneousProcessor, processor_interface::Processor},
    scheduler::{
        dag_set_scheduler::{DAGSetSchedulerBase, PreemptiveType},
        fixed_priority_scheduler::FixedPriorityScheduler,
        proposed_edf_scheduler::GlobalEDFScheduler,
    },
    task::{dag::DAG, dag_creator::create_dag_set_from_dir},
};

#[derive(Clone, ValueEnum)]
enum Algorithm {
    ProposedEDF,
    RM,
    Greedy,
}

#[derive(Parser)]
struct ArgParser {
    /// Path to DAGSet directory.
    #[clap(short = 'd', long = "dag_dir_path", default_value = "../autoware_dags")]
    dag_dir_path: String,
    /// Scheduling algorithm.
    #[clap(short = 'a', long = "algorithm", required = true, value_enum)]
    algorithm: Algorithm,
    /// Number of processing cores.
    #[clap(short = 'c', long = "number_of_cores", required = true)]
    number_of_cores: usize,
    /// Path to output directory.
    #[clap(short = 'o', long = "output_dir_path", default_value = "../outputs")]
    output_dir_path: String,
}

fn main() {
    let arg: ArgParser = ArgParser::parse();
    let mut dag_set = create_dag_set_from_dir(&arg.dag_dir_path);
    let homogeneous_processor = HomogeneousProcessor::new(arg.number_of_cores);

    match arg.algorithm {
        Algorithm::ProposedEDF => {
            let mut scheduler = GlobalEDFScheduler::new(&dag_set, &homogeneous_processor);
            scheduler.schedule(PreemptiveType::Preemptive {
                key: "ref_absolute_deadline".to_string(),
            });
            scheduler.dump_log(&arg.output_dir_path, "proposed_edf");
        }
        Algorithm::RM => {
            for dag in dag_set.iter_mut() {
                let dag_period = dag.get_head_period().unwrap();
                for node in dag.node_weights_mut() {
                    node.params.insert("priority".to_string(), dag_period);
                }
            }

            let mut scheduler = FixedPriorityScheduler::new(&dag_set, &homogeneous_processor);
            scheduler.schedule(PreemptiveType::Preemptive {
                key: "priority".to_string(),
            });
            scheduler.dump_log(&arg.output_dir_path, "rm");
        }
        Algorithm::Greedy => {
            let uniform_priority = 0;
            for dag in dag_set.iter_mut() {
                for node in dag.node_weights_mut() {
                    node.params.insert("priority".to_string(), uniform_priority);
                }
            }

            let mut scheduler = FixedPriorityScheduler::new(&dag_set, &homogeneous_processor);
            scheduler.schedule(PreemptiveType::NonPreemptive);
            scheduler.dump_log(&arg.output_dir_path, "greedy");
        }
    }
}
