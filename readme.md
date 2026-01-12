# megalib-py

[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

Python bindings for [**megalib**](https://crates.io/crates/megalib), a fast and robust Rust library for interacting with the Mega.nz API.

## Features

- **Fast & Async**: Built on `pyo3` and `tokio`, offering native async/await support for high-concurrency applications.
- **Complete Filesystem Operations**: `mkdir`, `list` (with recursive option), `mv`, `rename`, `rm`, `stat`.
- **Secure File Transfer**: High-speed, encrypted `upload` and `download` with automatic retry and resume support.
- **Account Management**: `login` (with proxy support), `register`, `quota`, `change_password`.
- **Session Caching**: `save` and `load` sessions to avoid re-authenticating.
- **Sharing**: Share folders with other users, list contacts, export public links.
- **Public Access**: Browse public folders and download public files without login.
- **Configuration**: Set parallel workers, enable resume, enable preview generation.
- **Type Hinted**: Includes full `.pyi` type stubs for excellent IDE support and autocompletion.

## Installation

```bash
pip install megalib
```

*Note: You may need to build from source if pre-built wheels are not available for your platform.*

### Build from Source

Requirements:
- Rust (latest stable)
- Python 3.8+

```bash
# Install maturin build system
pip install maturin

# Build and install
maturin develop --release

# Or install directly
pip install .
```

## Quick Start

```python
import asyncio
import os
from megalib import MegaSession

async def main():
    email = os.getenv("MEGA_EMAIL")
    password = os.getenv("MEGA_PASSWORD")
    
    # Login
    session = await MegaSession.login(email, password)
    await session.refresh()  # Load file tree
    
    # Check Storage
    total, used = await session.quota()
    print(f"Quota: {used / 1024**3:.2f} GB / {total / 1024**3:.2f} GB")

    # List Files
    files = await session.list("/Root")
    for f in files:
        print(f"{'📁' if f.is_folder else '📄'} {f.name}")

    # Upload a file
    await session.upload("local.txt", "/Root")
    
    # Save session for later
    await session.save(".mega_session.json")

if __name__ == "__main__":
    asyncio.run(main())
```

## API Reference

### `MegaSession`

The main entry point for interacting with your Mega account.

**Authentication:**
- `login(email, password, proxy=None) -> MegaSession`: Authenticate and start a session.
- `load(path) -> MegaSession | None`: Load a cached session from file.
- `save(path)`: Save session to file for later restoration.
- `refresh()`: Refresh the filesystem tree from the server.

**User Info:**
- `get_email() -> str`: Get user's email address.
- `get_name() -> str | None`: Get user's display name.
- `get_handle() -> str`: Get user's MEGA handle (unique ID).
- `quota() -> Tuple[int, int]`: Return `(total_bytes, used_bytes)`.

**Filesystem Operations:**
- `stat(path) -> MegaNode | None`: Get info about a file or folder.
- `list(path, recursive=False) -> List[MegaNode]`: List nodes in a folder.
- `mkdir(path)`: Create a new directory.
- `rename(path, new_name)`: Rename a file or folder.
- `mv(source, dest)`: Move a node to a new location.
- `rm(path)`: Delete a file or folder.

**File Transfer:**
- `upload(local_path, remote_path)`: Upload a file.
- `upload_resumable(local_path, remote_path)`: Upload with resume support.
- `download(remote_path, local_path)`: Download a file.
- `download_to_file(remote_path, local_path)`: Download with auto-resume.

**Sharing:**
- `export(path) -> str`: Generate a public download link.
- `share_folder(path, email, access_level)`: Share folder with another user (0=read, 1=write, 2=full).
- `list_contacts() -> List[MegaNode]`: List all contacts.

**Configuration:**
- `set_workers(count)`: Set number of parallel transfer workers.
- `set_resume(enabled)`: Enable/disable resume for interrupted transfers.
- `enable_previews(enabled)`: Enable/disable thumbnail generation on upload.
- `change_password(new_password)`: Change the user's password.

### `MegaNode`

Represents a file or folder in MEGA.

- `name: str`: File/folder name
- `handle: str`: Unique MEGA handle
- `size: int`: Size in bytes (0 for folders)
- `timestamp: int`: Unix timestamp of last modification
- `is_file: bool`: True if this is a file
- `is_folder: bool`: True if this is a folder

### `MegaPublicFolder`

For browsing public shared folders without login.

- `list(path) -> List[MegaNode]`: List files in the public folder.
- `download(remote_path, local_path)`: Download a file from the public folder.

### Global Functions

For operations that don't require an account session.

- `get_public_file_info(url) -> MegaPublicFile`: Get name and size of a public link.
- `download_public_file(url, local_path)`: Download a file directly from a public link.
- `open_folder(url) -> MegaPublicFolder`: Open a public folder for browsing.
- `register(email, password, name) -> MegaRegistrationState`: Start registration.
- `verify_registration(state, signup_key)`: Complete registration with key from email.

## Example Script

See [example.py](example.py) for a complete demonstration of all features.

```bash
# Set credentials
$env:MEGA_EMAIL='user@example.com'
$env:MEGA_PASSWORD='yourpassword'

# Run demo
python example.py
```

## License

This project is licensed under the GNU General Public License v2.0 (GPLv2) - see the [license](license) file for details.