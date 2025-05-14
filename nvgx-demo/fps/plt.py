import matplotlib.pyplot as plt
import numpy as np

def clean_data(data, sigma):
    means = np.nanmean(data, keepdims=True)
    stds = np.nanstd(data, keepdims=True)
    lower_bound = (means - sigma * stds)
    upper_bound = (means + sigma * stds)
    mask = (data >= lower_bound) & (data <= upper_bound)
    cleaned_data = data[mask]
    return cleaned_data


fig, ax = plt.subplots(nrows=1, ncols=1, figsize=(7, 5))

prefix = "nvgx-demo\\fps\\"

data_file = [
    "demo-cutout (OpenGL).csv",
    "demo-cutout-inst (OpenGL).csv",
    "demo-cutout (WGPU).csv",
    "demo-cutout-inst (WGPU).csv",
]

all_data = []
for file_name in data_file:
    data = np.loadtxt(prefix + file_name)
    data = clean_data(data, 3)
    all_data.append(data)

# Fixing random state for reproducibility
# np.random.seed(19680801)
# generate some random test data
# all_data = [np.random.normal(0, std, 100) for std in range(6, 10)]

palette = ["lightcoral", "lightcoral", "lightskyblue", "lightskyblue"]
# plot violin plot
vp = ax.violinplot(all_data, showmeans=False, showmedians=True)
for body, color in zip(vp['bodies'], palette):
    body.set_facecolor(color)
    body.set_edgecolor('gray')
    body.set_alpha(0.7)

ax.set_title("CUTOUT Bench Mark(CPU: 7940HS, GPU: 780M)")
ax.set_ybound(0, 3500)

# adding horizontal grid lines
ax.yaxis.grid(True)
ax.set_xticks(
    [y + 1 for y in range(len(all_data))],
    labels=["OpenGL", "OpenGL(Inst)", "WGPU-Vulkan", "WGPU-Vulkan(Inst)"],
)
ax.set_ylabel("FPS")

plt.show()
