#!/usr/bin/env python3
"""
Start a 3-node VectorXLite distributed cluster with Raft consensus

Port convention:
  Node 1: raft=5001, cluster=5002, vector=5003
  Node 2: raft=5011, cluster=5012, vector=5013
  Node 3: raft=5021, cluster=5022, vector=5023

Usage:
    python tools/start_cluster.py
"""

import subprocess
import sys
import time
import socket
from pathlib import Path
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn

console = Console()


def check_port(port: int, timeout: int = 30) -> bool:
    """Check if a port is available"""
    for _ in range(timeout):
        try:
            with socket.create_connection(("localhost", port), timeout=1):
                return True
        except (socket.timeout, ConnectionRefusedError, OSError):
            time.sleep(1)
    return False


def is_port_in_use(port: int) -> bool:
    """Check if a port is already in use"""
    try:
        with socket.create_connection(("localhost", port), timeout=1):
            return True
    except (ConnectionRefusedError, OSError):
        return False


def run_cargo_build(cwd: Path, args: list[str]) -> subprocess.Popen:
    """Run cargo command in background"""
    cmd = ["cargo", "run", "--release"] + args
    log_file = cwd / "logs" / f"{args[-1].split(':')[-1]}.log"
    log_file.parent.mkdir(parents=True, exist_ok=True)

    with open(log_file, 'w') as f:
        process = subprocess.Popen(
            cmd,
            cwd=cwd,
            stdout=f,
            stderr=subprocess.STDOUT,
            start_new_session=True
        )
    return process


def run_go_build(cwd: Path, binary: str, args: list[str]) -> subprocess.Popen:
    """Run Go binary in background"""
    cmd = [binary] + args
    log_file = cwd / "logs" / f"{args[1]}.log"  # args[1] is node ID
    log_file.parent.mkdir(parents=True, exist_ok=True)

    with open(log_file, 'w') as f:
        process = subprocess.Popen(
            cmd,
            cwd=cwd,
            stdout=f,
            stderr=subprocess.STDOUT,
            start_new_session=True
        )
    return process


def start_vector_servers(root_dir: Path, cluster_dir: Path) -> list[int]:
    """Start 3 VectorXLite servers"""
    console.print("[yellow]Starting VectorXLite servers...[/yellow]")

    server_dir = root_dir / "standalone" / "server"
    pids = []

    for i, port in enumerate([5003, 5013, 5023], 1):
        if is_port_in_use(port):
            console.print(f"  [green]VectorXLite server {i} already running on port {port}[/green]")
            continue

        console.print(f"  Starting VectorXLite server {i} on port {port}...")
        process = run_cargo_build(
            server_dir,
            ["--", "--port", str(port)]
        )
        pids.append(process.pid)
        console.print(f"    Node{i} VectorXLite started with PID: {process.pid}")

    # Save PIDs
    if pids:
        pid_file = cluster_dir / ".vector_xlite_pids"
        with open(pid_file, 'w') as f:
            for pid in pids:
                f.write(f"{pid}\n")

    # Wait for servers to be ready
    console.print("\n  Waiting for VectorXLite servers to be ready...")
    time.sleep(5)

    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console
    ) as progress:
        for port in [5003, 5013, 5023]:
            task = progress.add_task(f"Checking port {port}...", total=None)
            if check_port(port):
                progress.update(task, description=f"[green]✓ Port {port} ready[/green]")
                progress.stop_task(task)
            else:
                progress.update(task, description=f"[red]✗ Port {port} failed[/red]")
                progress.stop_task(task)
                console.print(f"[red]Error: VectorXLite server on port {port} failed to start[/red]")
                console.print(f"Check {cluster_dir}/logs/vector_xlite_node*.log for details")
                sys.exit(1)

    return pids


def build_binaries(cluster_dir: Path):
    """Build Go binaries"""
    console.print("\n[yellow]Building cluster binaries...[/yellow]")

    bin_dir = cluster_dir / "bin"
    bin_dir.mkdir(exist_ok=True)

    # Build server
    console.print("  Building server binary...")
    result = subprocess.run(
        ["go", "build", "-o", "bin/server", "cmd/server/main.go"],
        cwd=cluster_dir,
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        console.print(f"[red]Error building server: {result.stderr}[/red]")
        sys.exit(1)

    # Build CLI
    console.print("  Building CLI binary...")
    result = subprocess.run(
        ["go", "build", "-o", "bin/client", "cmd/cli/main.go"],
        cwd=cluster_dir,
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        console.print(f"[red]Error building CLI: {result.stderr}[/red]")
        sys.exit(1)

    console.print("  [green]✓ Binaries built successfully[/green]")


def start_cluster_nodes(cluster_dir: Path) -> list[int]:
    """Start 3 cluster nodes"""
    console.print("\n[yellow]Starting cluster nodes...[/yellow]")

    pids = []
    configs = [
        ("node1", "500", "0.0.0.0:5003", True),
        ("node2", "501", "0.0.0.0:5013", False),
        ("node3", "502", "0.0.0.0:5023", False),
    ]

    for node_id, port, vector_addr, bootstrap in configs:
        console.print(f"  Starting {node_id}{'(bootstrap)' if bootstrap else ''}...")

        args = [
            "-id", node_id,
            "-port", port,
            "-vector-addr", vector_addr,
            "-data-dir", "./data",
        ]
        if bootstrap:
            args.append("-bootstrap")

        process = run_go_build(cluster_dir, "./bin/server", args)
        pids.append(process.pid)
        console.print(f"    {node_id} started with PID: {process.pid}")
        time.sleep(2 if bootstrap else 1)

    # Save PIDs
    pid_file = cluster_dir / ".cluster_pids"
    with open(pid_file, 'w') as f:
        for pid in pids:
            f.write(f"{pid}\n")

    console.print("  [green]✓ All nodes started successfully![/green]")

    console.print("\n[cyan]Cluster configuration:[/cyan]")
    console.print("  Node1: raft=127.0.0.1:5001, cluster=:5002, vector=:5003")
    console.print("  Node2: raft=127.0.0.1:5011, cluster=:5012, vector=:5013")
    console.print("  Node3: raft=127.0.0.1:5021, cluster=:5022, vector=:5023")

    return pids


def join_cluster(cluster_dir: Path):
    """Join nodes to the cluster"""
    console.print("\n[yellow]Forming cluster...[/yellow]")
    console.print("  Waiting for cluster to stabilize (5s)...")
    time.sleep(5)

    # Join node2
    console.print("  Joining node2 to cluster...")
    result = subprocess.run(
        ["./bin/client", "join", "-addr", ":5002", "-node-id", "node2", "-node-addr", "127.0.0.1:5011"],
        cwd=cluster_dir,
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        console.print(f"  [yellow]Note: {result.stdout.strip()}[/yellow]")

    # Join node3
    console.print("  Joining node3 to cluster...")
    result = subprocess.run(
        ["./bin/client", "join", "-addr", ":5002", "-node-id", "node3", "-node-addr", "127.0.0.1:5021"],
        cwd=cluster_dir,
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        console.print(f"  [yellow]Note: {result.stdout.strip()}[/yellow]")

    time.sleep(2)


def show_cluster_info(cluster_dir: Path):
    """Show cluster information"""
    console.print("\n[green]=== Cluster Info ===[/green]")
    result = subprocess.run(
        ["./bin/client", "info", "-addr", ":5002"],
        cwd=cluster_dir,
        capture_output=True,
        text=True
    )
    console.print(result.stdout)


def main():
    root_dir = Path(__file__).parent.parent.parent.resolve()
    cluster_dir = root_dir / "distributed" / "cluster"

    if not cluster_dir.exists():
        console.print(f"[red]Error: Cluster directory not found: {cluster_dir}[/red]")
        sys.exit(1)

    console.print("[bold cyan]=== Starting VectorXLite Distributed Cluster ===[/bold cyan]\n")

    # Create necessary directories
    (cluster_dir / "data").mkdir(exist_ok=True)
    (cluster_dir / "logs").mkdir(exist_ok=True)

    try:
        # Start VectorXLite servers
        start_vector_servers(root_dir, cluster_dir)

        # Build Go binaries
        build_binaries(cluster_dir)

        # Start cluster nodes
        start_cluster_nodes(cluster_dir)

        # Join cluster
        join_cluster(cluster_dir)

        # Show cluster info
        show_cluster_info(cluster_dir)

        console.print("\n[bold green]✓ Cluster is ready for operations![/bold green]")
        console.print(f"\nLogs are in {cluster_dir}/logs/ directory")
        console.print("To stop cluster: python tools/stop_cluster.py")

    except KeyboardInterrupt:
        console.print("\n[yellow]Interrupted by user[/yellow]")
        sys.exit(1)
    except Exception as e:
        console.print(f"\n[red]Error: {e}[/red]")
        sys.exit(1)


if __name__ == "__main__":
    main()
