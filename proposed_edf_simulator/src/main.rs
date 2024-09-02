use clap::{Parser, ValueEnum};
use proposed_edf_simulator::{
    dag_creator::create_dag_set_from_dir,
    dag_set_scheduler::{DAGSetSchedulerBase, PreemptiveType},
    homogeneous::HomogeneousProcessor,
    processor::ProcessorBase,
    proposed_edf_scheduler::GlobalEDFScheduler,
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

    let dag_set = create_dag_set_from_dir(&arg.dag_dir_path);
    let homogeneous_processor = HomogeneousProcessor::new(arg.number_of_cores);
    let mut scheduler = match arg.algorithm {
        Algorithm::ProposedEDF => GlobalEDFScheduler::new(&dag_set, &homogeneous_processor),
        Algorithm::RM => todo!(),
        Algorithm::Greedy => todo!(),
    };

    // Change whether it is preemptive or not depending on the argument.
    let (preemptive_type, file_name) = match arg.algorithm {
        Algorithm::ProposedEDF => (
            PreemptiveType::Preemptive {
                key: "ref_absolute_deadline".to_string(),
            },
            "proposed_edf",
        ),
        Algorithm::RM => (
            PreemptiveType::Preemptive {
                key: "dag_period".to_string(),
            },
            "rm",
        ),
        Algorithm::Greedy => (PreemptiveType::NonPreemptive, "greedy"),
    };

    scheduler.schedule(preemptive_type);
    scheduler.dump_log(&arg.output_dir_path, file_name);
}
