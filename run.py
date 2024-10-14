from datetime import datetime
import subprocess
import os
import sys
import tomllib


def run_command(command):
    try:
        result = subprocess.run(command, check=True, text=True, capture_output=True)
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {' '.join(command)}")
        print(e.stderr)
        sys.exit(1)


def main():
    print("Running `cargo test`...")
    run_command(["cargo", "test"])

    print("Running `cargo build`...")
    run_command(["cargo", "build"])

    # Create output directory
    timestamp = datetime.now().strftime("%m%d%H%M")
    output_dir_path = os.path.join("outputs", timestamp)
    os.makedirs(output_dir_path, exist_ok=True)

    print(f"Output to {output_dir_path}")

    input_files_sorted = sorted(os.listdir("inputs"))

    with open("Cargo.toml", "rb") as f:
        data = tomllib.load(f)
        binary_path = f"./target/debug/{data['package']['name']}"

    for file_name in input_files_sorted:
        input_path = os.path.join("inputs", file_name)
        output_path = os.path.join(output_dir_path, file_name)

        print(f"Processing {file_name}...")

        try:
            with open(input_path, "r") as infile, open(output_path, "w") as outfile:
                result = subprocess.run(
                    [binary_path],
                    stdin=infile,
                    stdout=outfile,
                    stderr=subprocess.PIPE,
                    text=True,
                )

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
