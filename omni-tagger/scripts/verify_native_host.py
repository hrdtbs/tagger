import sys
import os
import struct
import json
import subprocess
import time

def verify_native_host():
    # Paths
    # Assuming running from repo root
    base_dir = os.path.abspath("omni-tagger/src-tauri/target/debug")
    native_host_path = os.path.join(base_dir, "native_host")
    mock_app_path = os.path.join(base_dir, "omni-tagger")
    log_path = "/tmp/omni_tagger_mock.log"

    if not os.path.exists(native_host_path):
        print(f"Error: native_host not found at {native_host_path}. Please build it first.")
        sys.exit(1)

    # Clean up
    if os.path.exists(log_path):
        os.remove(log_path)

    # Create Mock App
    with open(mock_app_path, "w") as f:
        f.write('#!/bin/bash\n')
        f.write(f'echo "Called with args: $@" > {log_path}\n')
    os.chmod(mock_app_path, 0o755)

    print(f"Created mock app at {mock_app_path}")

    # Prepare Message
    msg = {"url": "http://example.com/image.jpg", "data": None}
    msg_json = json.dumps(msg).encode('utf-8')
    length = len(msg_json)

    # Run Native Host
    print(f"Running native host: {native_host_path}")
    proc = subprocess.Popen(
        [native_host_path],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    # Send Message
    try:
        proc.stdin.write(struct.pack('@I', length))
        proc.stdin.write(msg_json)
        proc.stdin.flush()

        # Read Response
        len_bytes = proc.stdout.read(4)
        if not len_bytes:
            stderr = proc.stderr.read().decode()
            print(f"Error: No response length. Stderr: {stderr}")
            sys.exit(1)

        res_len = struct.unpack('@I', len_bytes)[0]
        res_bytes = proc.stdout.read(res_len)
        response = json.loads(res_bytes)

        print(f"Response: {response}")

    except Exception as e:
        print(f"Exception during communication: {e}")
        proc.kill()
        sys.exit(1)

    # Close
    proc.stdin.close()
    proc.terminate()
    try:
        proc.wait(timeout=1)
    except subprocess.TimeoutExpired:
        proc.kill()

    # Verify Mock Execution
    time.sleep(1) # Wait for fs
    if os.path.exists(log_path):
        with open(log_path, "r") as f:
            content = f.read().strip()
        print(f"Mock log content: {content}")
        if "--process-url http://example.com/image.jpg" in content:
            print("SUCCESS: Mock app called with correct arguments.")
        else:
            print("FAILURE: Mock app called with incorrect arguments.")
            sys.exit(1)
    else:
        print("FAILURE: Mock app was NOT called.")
        sys.exit(1)

if __name__ == "__main__":
    verify_native_host()
