# VectorXLite Tools

This directory contains automation tools for building, testing, and managing VectorXLite.

## Directory Structure

```
tools/
├── python/          # Python automation scripts (recommended)
│   ├── generate_protos.py
│   ├── start_cluster.py
│   ├── stop_cluster.py
│   ├── test_operations.py
│   ├── test_all_modes.py
│   └── setup.py
├── shell/           # Shell scripts (legacy reference)
│   ├── generate-protos.sh
│   ├── start_cluster.sh
│   ├── stop_cluster.sh
│   ├── test_operations.sh
│   └── test_all_modes.sh
├── docs/            # Documentation
│   ├── README.md (this file)
│   └── TEST_RESULTS.md
├── venv/            # Python virtual environment
├── requirements.txt
└── .gitignore
```

## Contents

### Python Scripts (Recommended)
Located in `tools/python/`:
- **`generate_protos.py`** - Generate Go protobuf files from .proto definitions
- **`start_cluster.py`** - Start a 3-node distributed cluster with Raft consensus
- **`stop_cluster.py`** - Stop the distributed cluster
- **`test_operations.py`** - Test distributed cluster operations
- **`test_all_modes.py`** - Test all three deployment modes (embedded, standalone, distributed)
- **`setup.py`** - Setup virtual environment and install dependencies

### Shell Scripts (Legacy Reference)
Located in `tools/shell/`:
- **`generate-protos.sh`** - Shell version of proto generation
- **`start_cluster.sh`** - Shell version of cluster startup
- **`stop_cluster.sh`** - Shell version of cluster shutdown
- **`test_operations.sh`** - Shell version of operations testing
- **`test_all_modes.sh`** - Shell version of all-modes testing

## Setup

### 1. Install Python Dependencies

**Option A: Using the setup script (Recommended)**
```bash
python tools/python/setup.py
```

**Option B: Manual setup**
```bash
cd tools
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
```

### 2. Prerequisites

Ensure you have installed:
- **Python 3.8+**
- **Protocol Buffer Compiler (protoc)**
  ```bash
  # Ubuntu/Debian
  sudo apt-get install protobuf-compiler

  # macOS
  brew install protobuf
  ```
- **Go protobuf plugins**
  ```bash
  go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
  go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
  ```

## Usage

### Generate Protocol Buffers

```bash
# Using Python (recommended)
python tools/python/generate_protos.py

# Or with venv
tools/venv/bin/python tools/python/generate_protos.py

# Legacy shell version
bash tools/shell/generate-protos.sh
```

### Distributed Cluster Management

**Start the cluster:**
```bash
python tools/python/start_cluster.py
```

This will:
1. Start 3 VectorXLite gRPC servers (ports 5003, 5013, 5023)
2. Build cluster binaries (server and CLI)
3. Start 3 cluster nodes with Raft consensus
4. Form the cluster and show status

**Test cluster operations:**
```bash
python tools/python/test_operations.py
```

Tests include:
- Cluster info retrieval
- Collection creation
- Vector insertion
- Vector search
- Follower reads
- Leader redirect

**Stop the cluster:**
```bash
# Stop cluster nodes only
python tools/python/stop_cluster.py

# Stop cluster nodes and VectorXLite servers
python tools/python/stop_cluster.py --with-vector-server
```

### Test All Modes

```bash
python tools/python/test_all_modes.py
```

This tests:
1. **Embedded Mode** - Direct Rust library usage
2. **Standalone Mode** - gRPC server with Go client
3. **Distributed Mode** - Shows manual test instructions

## Port Configuration

The distributed cluster uses the following ports:

| Node   | Raft Port | Cluster gRPC | Vector Server |
|--------|-----------|--------------|---------------|
| Node 1 | 5001      | 5002         | 5003          |
| Node 2 | 5011      | 5012         | 5013          |
| Node 3 | 5021      | 5022         | 5023          |

## Logs

Cluster logs are stored in:
- `distributed/cluster/logs/` - Cluster node logs
- `distributed/cluster/data/` - Raft data directories

## Troubleshooting

### Port Already in Use
```bash
# Check what's using a port
lsof -i :5002

# Kill processes on a specific port
kill $(lsof -t -i:5002)
```

### Proto Generation Fails
- Ensure `protoc` is installed: `protoc --version`
- Ensure Go plugins are installed and in PATH:
  ```bash
  which protoc-gen-go
  which protoc-gen-go-grpc
  ```

### Cluster Won't Start
- Check logs in `distributed/cluster/logs/`
- Ensure VectorXLite servers are running
- Verify ports are not in use

## Development

### Adding New Tools

1. **Python scripts**: Create in `tools/python/` directory
2. **Shell scripts**: Add to `tools/shell/` directory (for reference)
3. **Documentation**: Update `tools/docs/README.md`
4. Add any new dependencies to `tools/requirements.txt`
5. Make scripts executable: `chmod +x tools/python/your_script.py`

### Python Script Template

```python
#!/usr/bin/env python3
"""
Tool description

Usage:
    python tools/python/your_tool.py
"""

from pathlib import Path
from rich.console import Console

console = Console()

def main():
    # Root is two levels up (tools/python/ -> tools/ -> root/)
    root_dir = Path(__file__).parent.parent.parent.resolve()
    console.print("[cyan]Your tool is running...[/cyan]")

if __name__ == "__main__":
    main()
```

## Migration from Shell Scripts

All shell scripts have been converted to Python for better:
- Cross-platform compatibility
- Error handling
- Output formatting (via Rich library)
- Process management (via psutil)

The original shell scripts are kept for reference but Python scripts are recommended for all operations.
