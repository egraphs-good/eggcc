import sys

def process_file(filename):
    try:
        # Read the file contents
        with open(filename, 'r') as file:
            numbers = file.readlines()

        # Convert to float, round to two decimal places, and format as strings
        rounded_numbers = [f"{float(num.strip()):.2f}" for num in numbers if num.strip()]

        # Group numbers into lines of 20
        formatted_lines = [" ".join(rounded_numbers[i:i+20]) + " " for i in range(0, len(rounded_numbers), 20)]

        # Write the processed numbers back to the file
        with open(filename, 'w') as file:
            file.write("\n".join(formatted_lines) + "\n")
    
    except Exception as e:
        print(f"Error processing file {filename}: {e}")
        sys.exit(1)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python3 post.py <filename>")
        sys.exit(1)
    
    process_file(sys.argv[1])
