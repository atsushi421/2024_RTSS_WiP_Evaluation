#!/bin/bash

NUM_CORES=4
SIM_DURATION=1000000

cd /home/atsushi/2024_RTSS_WiP_Evaluation/scheduling_simulator
cargo build --release
for alg in "${ALGS[@]}"
do
    RUST_BACKTRACE=1 /home/atsushi/2024_RTSS_WiP_Evaluation/scheduling_simulator/target/release/scheduling_simulator -c $NUM_CORES -s $SIM_DURATION
done
