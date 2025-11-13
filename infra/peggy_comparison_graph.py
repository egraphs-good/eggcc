import pandas as pd
import matplotlib
import matplotlib.pyplot as plt
import csv
import json

def get_eggcc_df(profile_data):
    result = []
    for run in profile_data:
        if run['runMethod'] == 'eggcc-tiger-O0-O0' or run['runMethod'] == 'eggcc-tiger-ILP-O0-O0':
            bril_file = run['path']
            eggcc_time = run['eggccCompileTimeSecs']
            extraction_time = run['eggccExtractionTimeSecs']
            with open(bril_file, 'r') as file:
                line_count = sum(1 for line in file 
                                 if line.strip() and not line.strip().startswith('#'))
            result.append([run['runMethod'], line_count, eggcc_time, extraction_time])
    df = pd.DataFrame(result, columns=['runMethod', 'length','compile','extraction'])
    return df

def make_peggy_comparison_graph(eggcc_profile, peggy_file, eggcc_figure, peggy_figure):
    eggcc_data = get_eggcc_df(eggcc_profile)
    # Set plotting parameters
    plt.rcParams["font.size"] = 16
    transparency = 0.2
    size = 150

    peggy_data = clean_data(peggy_file)
    
    # Create a single figure with 4 subfigures
    fig, axs = plt.subplots(figsize=(8, 8))
    
    eggcc_figure
    
    # Second row: Ratio plots (split)
    # Peggy ratio
    ratio_plot_single(
        peggy_data,
        transparency,
        size,
        num="PBTIME",
        denom="PEG2PEGTIME",
        xcol="length",
        xlabel="Number of Java bytecode instructions",
        ylabel="Extraction runtime percentage",
        ax=axs,
        color="green",
        label="Peggy ILP"
    )
    axs.set_title("ILP runtime percentage in Peggy")
    plt.tight_layout()
    fig.savefig(peggy_figure, bbox_inches="tight")
    
    fig, axs = plt.subplots(figsize=(8, 8))
    
    # EGGCC ratio
    ratio_plot_single_eggcc(
        eggcc_data,
        transparency,
        size,
        num="extraction",
        denom="compile",
        xcol="length",
        xlabel="Number of Bril instructions",
        ylabel="Extraction runtime percentage",
        ax=axs,
        eggcc_color="blue",
        ilp_color="green",
        eggcc='eggcc-tiger-O0-O0',
        ilp='eggcc-tiger-ILP-O0-O0',
        label_eggcc="Statewalk DP",
        label_ilp="ILP",
    )
    axs.set_title("Statewalk DP runtime percentage in EQCC")
    
    # Adjust layout and save
    plt.tight_layout()
    fig.savefig(eggcc_figure, bbox_inches="tight")
    plt.close(fig)


TIMEOUT = -1
FAILURE = -2


def clean_data(results_file):
    data = pd.read_csv(results_file)

    # strip spaces from each column name
    data.columns = data.columns.str.strip()

    data = data[data["length"] < 500]
    # scale to seconds,
    # indicate timeout and failure by -1 or -2
    for timecol in ["PEG2PEGTIME", "PBTIME", "ENGINETIME", "Optimization took"]:
        data[timecol] = data[timecol].apply(
            lambda x: (
                TIMEOUT
                if x == "TIMEOUT"
                else (FAILURE if x == "FAILURE" else int(x) / 1000)
            )
        )

    return data

def ratio_plot_single(data, transparency, size, num, denom, xcol, xlabel, ylabel, ax, color, label):
    ax.set_xlim(0, 400)
    ax.set_ylim(-0.04, 1.04)
    ax.set_xlabel(xlabel)
    ax.set_ylabel(ylabel)
    
    # Filter out failure and timeout entries
    filtered_data = data[(data[num] != FAILURE) & (data[num] != TIMEOUT) & 
                         (data[denom] != FAILURE) & (data[denom] != TIMEOUT)]
    # timeouts = data[(data[num] == FAILURE) | (data[num] == TIMEOUT) |
    #                      (data[denom] == FAILURE) | (data[denom] == TIMEOUT)]
    
    timeouts = data[(data[num] == TIMEOUT) | (data[denom] == TIMEOUT)]
    
    # Calculate ratio
    ratios = filtered_data[num] / filtered_data[denom]
    
    ax.scatter(filtered_data[xcol], ratios, c=color, s=size, alpha=transparency, label=label)
    ax.scatter(timeouts[xcol], [1] * len(timeouts), c='red', s=size, label=label + ' (timeout)', marker='x')
    ax.legend()


def ratio_plot_single_eggcc(data, transparency, size, num, denom, xcol, xlabel, ylabel, ax, eggcc_color, ilp_color, label_eggcc, label_ilp, eggcc, ilp):
    ax.set_xlim(0, 400)
    ax.set_ylim(-0.04, 1.04)
    ax.set_xlabel(xlabel)
    ax.set_ylabel(ylabel)
    
    # Calculate ratio
    # eggcc_data = data[data['runMethod']==eggcc]
    eggcc_data = data[(data['runMethod']== eggcc) & (data[num] != False) & (data[denom] != False)]
    ilp_data =  data[(data['runMethod']== ilp) & (data[num] != False) & (data[denom] != False)]
    ilp_timeout = data[(data[num] == False) | (data[denom] == False)]
    print(ilp_timeout)
    eggcc_ratios = pd.to_numeric(eggcc_data[num]) / pd.to_numeric(eggcc_data[denom])
    ilp_ratios = pd.to_numeric(ilp_data[num]) / pd.to_numeric(ilp_data[denom])
    
    ax.scatter(eggcc_data[xcol], eggcc_ratios, c=eggcc_color, s=size, alpha=transparency, label=label_eggcc)
    ax.scatter(ilp_data[xcol], ilp_ratios, c=ilp_color, s=size, alpha=transparency, label=label_ilp)
    ax.scatter(ilp_timeout[xcol], [1]*len(ilp_timeout), c='red', s=size, marker='x', label = label_ilp + ' (timeout)')
    ax.legend()


# if __name__ == "__main__":
#     make_peggy_comparison_graph("peggy_data.csv", "eggcc-extraction-time-ratio.pdf", "peggy-extraction-time-ratio.pdf")
