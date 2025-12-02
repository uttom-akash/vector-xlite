#!/usr/bin/env python3
"""
Stop the VectorXLite distributed cluster

Usage:
    python tools/stop_cluster.py [--with-vector-server]
"""

import argparse
import sys
from pathlib import Path
import psutil
from rich.console import Console

console = Console()


def read_pids_from_file(pid_file: Path) -> list[int]:
    """Read PIDs from a file"""
    if not pid_file.exists():
        return []

    try:
        with open(pid_file, 'r') as f:
            pids = [int(line.strip()) for line in f if line.strip()]
        return pids
    except (ValueError, IOError) as e:
        console.print(f"[yellow]Warning: Error reading {pid_file}: {e}[/yellow]")
        return []


def kill_process(pid: int) -> bool:
    """Kill a process by PID"""
    try:
        process = psutil.Process(pid)
        if process.is_running():
            console.print(f"  Stopping process (PID: {pid})...")
            process.terminate()
            try:
                process.wait(timeout=5)
            except psutil.TimeoutExpired:
                console.print(f"  [yellow]Process {pid} didn't terminate, killing forcefully[/yellow]")
                process.kill()
            return True
        else:
            console.print(f"  [dim]Process {pid} is not running[/dim]")
            return False
    except psutil.NoSuchProcess:
        console.print(f"  [dim]Process {pid} not found[/dim]")
        return False
    except psutil.AccessDenied:
        console.print(f"  [red]Access denied to process {pid}[/red]")
        return False


def kill_processes_by_name(name_patterns: list[str]) -> int:
    """Kill processes matching any of the name patterns"""
    killed = 0
    for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
        try:
            cmdline = ' '.join(proc.info['cmdline'] or [])
            if any(pattern in cmdline for pattern in name_patterns):
                console.print(f"  Found matching process: PID {proc.info['pid']}")
                if kill_process(proc.info['pid']):
                    killed += 1
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass
    return killed


def stop_cluster_nodes(cluster_dir: Path) -> bool:
    """Stop cluster nodes"""
    console.print("[yellow]Stopping VectorXLite Cluster nodes...[/yellow]")

    cluster_pid_file = cluster_dir / ".cluster_pids"

    if cluster_pid_file.exists():
        pids = read_pids_from_file(cluster_pid_file)
        if pids:
            for pid in pids:
                kill_process(pid)
            cluster_pid_file.unlink()
            console.print("[green]✓ Cluster nodes stopped successfully[/green]")
            return True
        else:
            console.print("[yellow]No PIDs found in cluster PID file[/yellow]")
    else:
        console.print("[dim]No cluster PIDs file found[/dim]")

    # Try to kill by process name
    console.print("  Attempting to kill cluster processes by name...")
    patterns = ["cmd/server/main.go", "bin/server"]
    killed = kill_processes_by_name(patterns)

    if killed > 0:
        console.print(f"[green]✓ Killed {killed} cluster processes[/green]")
        return True
    else:
        console.print("[dim]No cluster processes found[/dim]")
        return False


def stop_vector_servers(cluster_dir: Path) -> bool:
    """Stop VectorXLite servers"""
    console.print("[yellow]Stopping VectorXLite servers...[/yellow]")

    vector_pid_file = cluster_dir / ".vector_xlite_pids"

    if vector_pid_file.exists():
        pids = read_pids_from_file(vector_pid_file)
        if pids:
            for pid in pids:
                kill_process(pid)
            vector_pid_file.unlink()
            console.print("[green]✓ VectorXLite servers stopped[/green]")
            return True
        else:
            console.print("[yellow]No PIDs found in VectorXLite PID file[/yellow]")
    else:
        console.print("[dim]No VectorXLite PIDs file found[/dim]")

    # Try to kill by process name
    console.print("  Attempting to kill VectorXLite processes by name...")
    patterns = ["vector_xlite_grpc", "standalone/server"]
    killed = kill_processes_by_name(patterns)

    if killed > 0:
        console.print(f"[green]✓ Killed {killed} VectorXLite processes[/green]")
        return True
    else:
        console.print("[dim]No VectorXLite processes found[/dim]")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Stop the VectorXLite distributed cluster"
    )
    parser.add_argument(
        "--with-vector-server",
        action="store_true",
        help="Also stop VectorXLite servers"
    )

    args = parser.parse_args()

    # Get cluster directory
    root_dir = Path(__file__).parent.parent.parent.resolve()
    cluster_dir = root_dir / "distributed" / "cluster"

    if not cluster_dir.exists():
        console.print(f"[red]Error: Cluster directory not found: {cluster_dir}[/red]")
        sys.exit(1)

    # Stop cluster nodes
    stop_cluster_nodes(cluster_dir)

    # Stop VectorXLite servers if requested
    if args.with_vector_server:
        stop_vector_servers(cluster_dir)
    else:
        console.print("[yellow]VectorXLite servers left running (use --with-vector-server to stop them)[/yellow]")

    console.print("\n[green]✓ Done![/green]")


if __name__ == "__main__":
    main()
