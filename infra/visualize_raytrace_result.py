# Use this file on the output of `raytracewithscreen.rs` to visualize the output of the ray tracing algorithm.

import numpy as np
from PIL import Image

# Function to read 100x100 floating-point numbers
def read_floats():
    floats = []
    for i in range(100):
        row = []
        for j in range(100):
            number = float(input(f""))
            row.append(number)
        floats.append(row)
    return np.array(floats)

# Function to normalize values to the range [0, 255]
def normalize(floats):
    min_val = np.min(floats)
    max_val = np.max(floats)
    
    # Normalize the values to fit into the 0-255 range for grayscale
    norm_floats = (255 * (floats - min_val) / (max_val - min_val)).astype(np.uint8)
    return norm_floats

# Function to visualize the matrix as a grayscale image using PIL
def visualize(floats):
    # Create a PIL image from the normalized array
    img = Image.fromarray(floats, mode='L')  # 'L' mode is for grayscale

    # write the image to a file
    img.save("output.png")
    #img.show()

# Main function
def main():
    # Step 1: Read 100x100 floating-point numbers from input
    floats = read_floats()

    # Step 2: Normalize the values to a 0-255 range
    normalized_floats = normalize(floats)

    # Step 3: Visualize the 100x100 grid as grayscale
    visualize(normalized_floats)

if __name__ == "__main__":
    main()
