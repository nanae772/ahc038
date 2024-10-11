import subprocess
import os
import sys

def run_command(command):
    try:
        result = subprocess.run(command, check=True, text=True, capture_output=True)
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {' '.join(command)}")
        print(e.stderr)
        sys.exit(1)

def main():
    # Step 1: Run `cargo test`
    print("Running `cargo test`...")
    run_command(["cargo", "test"])
    
    # Step 2: Run `cargo build`
    print("Running `cargo build`...")
    run_command(["cargo", "build"])
    
    # Get the built binary path (assuming it's in target/debug/)
    binary_path = "./target/debug/ahc038"
    
    # Ensure the outputs directory exists
    os.makedirs("outputs", exist_ok=True)
    
    # Step 3: Iterate over files in the inputs directory
    for file_name in sorted(os.listdir("inputs")):
        input_path = os.path.join("inputs", file_name)
        output_path = os.path.join("outputs", file_name)
        
        print(f"Processing {file_name}...")
        
        try:
            with open(input_path, "r") as infile, open(output_path, "w") as outfile:
                result = subprocess.run([binary_path], stdin=infile, stdout=outfile, stderr=subprocess.PIPE, text=True)
                
                if result.returncode != 0:
                    print(f"Error executing {binary_path} with {file_name}")
                    print(result.stderr)
                    sys.exit(1)
        
        except Exception as e:
            print(f"An unexpected error occurred: {e}")
            sys.exit(1)

    print("All files processed successfully.")

if __name__ == "__main__":
    main()

