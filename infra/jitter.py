import json
import random
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker

# Read profile.json from nightly/output/data/profile.json
profile = []
with open('nightly/output/data/profile.json') as f:
    profile = json.load(f)

# Prepare the data for the jitter plot
x_labels = []
x_data = []
y_data = []
colors = []
color_map = {}
next_color = 0

filtered = profile
# filter to benchmarks with average cycles below 200
filtered = [b for b in filtered if b.get('cycles') and sum(b['cycles']) / len(b['cycles']) < 200]
filtered_names = [b.get('benchmark', '') for b in filtered]
filtered = [b for b in profile if b.get('benchmark', '') in filtered_names]

# Sort benchmarks by name
filtered = sorted(filtered, key=lambda b: b.get('benchmark', ''))

# Assign numeric x values to each benchmark label
x_label_map = {}
outlier_x = []
outlier_y = []

for idx, benchmark in enumerate(filtered):
    benchmark_name = benchmark.get('benchmark', f'benchmark_{idx}')
    name_and_treatment = benchmark_name + '_' + benchmark.get('runMethod', '')
    assert(name_and_treatment not in x_label_map)
    x_label_map[name_and_treatment] = idx
    x_labels.append(name_and_treatment)

    # Assign color for each runMethod
    if 'runMethod' not in benchmark:
        raise KeyError(f"Missing 'runMethod' field in benchmark: {benchmark_name}")
    run_method = benchmark['runMethod']
    if run_method not in color_map:
        color_map[run_method] = f'C{next_color}'
        next_color += 1
    color = color_map[run_method]

    for cycle in benchmark.get('cycles', [])[:100]:
        # Add a small random jitter to x value to prevent overlap
        jittered_x = idx + random.uniform(-0.2, 0.2)
        if cycle > 2000:
            # Record outlier data
            outlier_x.append(jittered_x)
            outlier_y.append(2000)
        else:
            # Normal data points
            x_data.append(jittered_x)
            y_data.append(cycle)
            colors.append(color)

# Create the jitter plot
plt.figure(figsize=(12, 6))
plt.scatter(x_data, y_data, c=colors, alpha=0.7, edgecolors='w', linewidth=0.5)

# Plot outliers as red 'x' marks
plt.scatter(outlier_x, outlier_y, color='red', marker='x', s=50, label='Outliers (> 200 cycles)', alpha=0.9)

# Set the labels and title
plt.xticks(range(len(x_labels)), x_labels, rotation=45, ha='right')
plt.xlabel('Benchmark')
plt.ylabel('Cycles')
plt.title('Jitter Plot of Benchmarks and Cycles')

# Set y-axis to display numbers instead of scientific notation
plt.gca().yaxis.set_major_formatter(mticker.FuncFormatter(lambda x, _: f'{int(x)}'))

# Create a legend based on runMethod
handles = [plt.Line2D([0], [0], marker='o', color='w', markerfacecolor=color_map[rm], markersize=10, alpha=0.7) for rm in color_map]
handles.append(plt.Line2D([0], [0], marker='x', color='red', markersize=10, linestyle='None', label='Outliers (> 200 cycles)'))
plt.legend(handles, list(color_map.keys()) + ['Outliers (> 200 cycles)'], title='Run Method', loc='upper right')

# Save the plot to a PNG file in the nightly directory
plt.tight_layout()
plt.savefig('nightly/jitter_plot.png')
