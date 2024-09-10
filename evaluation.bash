#!/bin/bash

NUM_CORES=7
SIM_DURATION=3000000
NUM_SIM=1000

cd /home/atsushi/2024_RTSS_WiP_Evaluation/scheduling_simulator
cargo build --release
export RAYON_NUM_THREADS=24
/home/atsushi/2024_RTSS_WiP_Evaluation/scheduling_simulator/target/release/scheduling_simulator -c $NUM_CORES -s $SIM_DURATION -r $NUM_SIM
