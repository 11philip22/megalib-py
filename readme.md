# megalib-py

High-performance Python bindings for **megalib**, a fast and robust Rust library for interacting with the Mega.nz API.

## Features

- **Fast & Async**: Built on `pyo3` and `tokio`, offering native async/await support for high-concurrency applications.
- **Complete Filesystem Operations**: `mkdir`, `list`, `mv`, `rename`, `rm`, `stat` (via list).
- **Secure File Transfer**: High-speed, encrypted `upload` and `download` with automatic retry logic.
- **Account Management**: `login`, `register`, `quota` checks.
- **Sharing**: Export public links and fetch info from public links.
- **Type Hinted**: Includes full `.pyi` type stubs for excellent IDE support and autocompletion.

## Installation

```bash
pip install megalib
```

*Note: You may need to build from source if pre-built wheels are not available for your platform.*

### Build from Source

Requirements:
- Rust (latest stable)
- Python 3.7+

```bash
# Install maturin build system
pip install maturin

# Build and install
maturin develop --release
```

## Quick Start
Set up your credentials and run a simple script:

```python
import asyncio
import os
from megalib import MegaSession

async def main():
    email = os.getenv("MEGA_EMAIL")
    password = os.getenv("MEGA_PASSWORD")
    
    # Login
    print("Logging in...")
    try:
        session = await MegaSession.login(email, password)
    except ValueError:
        print("Login failed! Check credentials.")
        return

    # Check Storage
    total, used = await session.quota()
    print(f"Quota: {used / 1024**3:.2f} GB / {total / 1024**3:.2f} GB")

    # List Files in Root
    files = await session.list("/Root")
    for f in files:
        print(f"{'[DIR] ' if f.is_folder else '[FILE]'} {f.name}")

    # Upload a file
    await session.upload("local_report.pdf", "/Root/Reports/report.pdf")
    print("Upload complete!")

if __name__ == "__main__":
    asyncio.run(main())
```

## API Reference

### `MegaSession`
The main entry point for interacting with your specific Mega account.

*   `static login(email: str, password: str) -> MegaSession`: Authenticate and start a session.
*   `list(path: str) -> List[MegaNode]`: List nodes in a folder.
*   `mkdir(path: str)`: Create a new directory.
*   `upload(local_path: str, remote_path: str)`: Upload a file to a specific remote path.
*   `download(remote_path: str, local_path: str)`: Download a file from Mega to your local machine.
*   `rename(path: str, new_name: str)`: Rename a file or folder.
*   `mv(source: str, dest: str)`: Move a node to a new location.
*   `rm(path: str)`: Delete a node (moves to Recycle Bin usually).
*   `export(path: str) -> str`: Generate a public download link for a node.
*   `quota() -> Tuple[int, int]`: Return `(total_bytes, used_bytes)`.

### Global Functions
For operations that don't require an account session.

*   `get_public_file_info(url: str) -> MegaPublicFile`: Get name and size of a public link.
*   `download_public_file(url: str, local_path: str)`: Download a file directly from a public link.
*   `register(email: str, password: str, name: str) -> MegaRegistrationState`: Start registration.
*   `verify_registration(state: MegaRegistrationState, signup_key: str)`: Complete registration with key from email.
