use scheduling_simulator::{
    processor::{homogeneous::HomogeneousProcessor, processor_interface::Processor},
    scheduler::{
        dag_set_scheduler::{DAGSetSchedulerBase, PreemptiveType},
        proposed_edf_scheduler::GlobalEDFScheduler,
    },
};

mod util;
use util::{create_multi_sink_dag_set, create_sequential_dag_set0};

#[test]
fn test_sequential_edf_scheduler() {
    let dag_set = create_sequential_dag_set0();
    let processor = HomogeneousProcessor::new(1);
    let mut scheduler = GlobalEDFScheduler::new(&dag_set, &processor);
    scheduler.schedule(
        PreemptiveType::Preemptive {
            key: "ref_absolute_deadline".to_string(),
        },
        26,
    );

    let log = scheduler.get_log();
    assert!(!log.deadline_missed);
    let rt0 = log.dag_set_log[0].response_times_per_sink[&0].clone();
    assert_eq!(rt0, vec![2, 3, 4, 2, 2]);
    let rt1 = log.dag_set_log[1].response_times_per_sink[&0].clone();
    assert_eq!(rt1, vec![6, 5, 6, 5]);
}

#[test]
fn test_multi_sink_dag_set_edf_scheduler() {
    let dag_set = create_multi_sink_dag_set();
    let processor = HomogeneousProcessor::new(1);
    let mut scheduler = GlobalEDFScheduler::new(&dag_set, &processor);
    scheduler.schedule(
        PreemptiveType::Preemptive {
            key: "ref_absolute_deadline".to_string(),
        },
        30,
    );

    let log = scheduler.get_log();
    assert!(!log.deadline_missed);
    assert_eq!(
        log.dag_set_log[0].response_times_per_sink[&1],
        vec![5, 6, 5]
    );
    assert_eq!(
        log.dag_set_log[0].response_times_per_sink[&2],
        vec![3, 4, 3]
    );
    assert_eq!(log.dag_set_log[1].response_times_per_sink[&1], vec![9, 5]);
    assert_eq!(log.dag_set_log[1].response_times_per_sink[&2], vec![11, 12]);
}
