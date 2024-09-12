# 2024_RTSS_WiP_Evaluation

## Setup

Please install Rust tool chain and jupyter-notebook:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
python3 -m pip install jupyterlab
```

## Run Simulation

The following command simulates the extended EDF, RM and work-conserving algorithms 5,000 times using the Autoware pseudo workload.

```bash
bash evaluation.bash
```

As this takes several hours to complete, this step can be skipped using the data we have obtained.

## Visualize Result

1. `jupyter-lab`
2. Open `visualize_result.ipynb` from left side bar.
3. Run all cells

> [!NOTE]
> Since our simulations are based on randomly selected execution times, the generated figure may differ slightly from the figure in the paper, even though the trend remains the same.
