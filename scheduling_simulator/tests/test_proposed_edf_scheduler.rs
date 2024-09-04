use scheduling_simulator::{
    processor::{homogeneous::HomogeneousProcessor, processor_interface::Processor},
    scheduler::{
        dag_set_scheduler::{DAGSetSchedulerBase, PreemptiveType},
        proposed_edf_scheduler::GlobalEDFScheduler,
    },
};

mod util;
use util::create_sequential_dag_set;

#[test]
fn test_sequential_edf_scheduler() {
    let dag_set = create_sequential_dag_set();
    let processor = HomogeneousProcessor::new(1);
    let mut scheduler = GlobalEDFScheduler::new(&dag_set, &processor);
    scheduler.schedule(
        PreemptiveType::Preemptive {
            key: "ref_absolute_deadline".to_string(),
        },
        26,
    );

    let log = scheduler.get_log();

    let mut rt0 = log.dag_set_log[0].response_times.clone();
    // assert_eq!(rt0, vec![2, 3, 4, 2, 2]);
    let mut rt1 = log.dag_set_log[1].response_times.clone();
    assert_eq!(rt1, vec![6, 5, 6, 5]);
}
