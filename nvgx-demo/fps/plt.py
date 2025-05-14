import matplotlib.pyplot as plt
import numpy as np

fig, ax = plt.subplots(nrows=1, ncols=1, figsize=(9, 4))

# Fixing random state for reproducibility
np.random.seed(19680801)

data_file = [
    "nvgx-demo\\fps\\demo-cutout (OpenGL).csv",
    "nvgx-demo\\fps\\demo-cutout-inst (OpenGL).csv",
    "nvgx-demo\\fps\\demo-cutout (WGPU).csv",
    "nvgx-demo\\fps\\demo-cutout-inst (WGPU).csv",
]

all_data = []
for file_name in data_file:
    data = np.loadtxt(file_name)
    all_data.append(data)

# generate some random test data
# all_data = [np.random.normal(0, std, 100) for std in range(6, 10)]


# plot violin plot
ax.violinplot(all_data, showmeans=False, showmedians=True)
ax.set_title("NVGX Bench Mark(CPU: 7940HS, GPU: 780M)")


# adding horizontal grid lines
ax.yaxis.grid(True)
ax.set_xticks(
    [y + 1 for y in range(len(all_data))],
    labels=["OpenGL", "OpenGL(Inst)", "WGPU-Vulkan", "WGPU-Vulkan(Inst)"],
)
ax.set_ylabel("FPS")

plt.show()
