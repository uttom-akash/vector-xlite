#!/usr/bin/env python3
"""
Generate Go protobuf files for VectorXLite

Prerequisites:
  - protoc (Protocol Buffer compiler)
  - protoc-gen-go: go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
  - protoc-gen-go-grpc: go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

Run this from the vector-db-rs root directory:
  python tools/generate_protos.py
"""

import subprocess
import sys
from pathlib import Path
from rich.console import Console
from rich.panel import Panel

console = Console()


def run_command(cmd: list[str], cwd: Path = None) -> tuple[int, str, str]:
    """Run a command and return exit code, stdout, stderr"""
    result = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True
    )
    return result.returncode, result.stdout, result.stderr


def generate_vectorxlite_protos(root_dir: Path) -> bool:
    """Generate VectorXLite gRPC protos (standalone client)"""
    console.print("\n[yellow]Generating VectorXLite protos...[/yellow]")

    proto_file = root_dir / "proto" / "vectorxlite" / "v1" / "vectorxlite.proto"
    output_dir = root_dir / "standalone" / "clients" / "go" / "pb"

    if not proto_file.exists():
        console.print(f"[red]Error: Proto file not found: {proto_file}[/red]")
        return False

    output_dir.mkdir(parents=True, exist_ok=True)

    cmd = [
        "protoc",
        f"--proto_path={root_dir / 'proto'}",
        f"--go_out=paths=source_relative:{output_dir}",
        f"--go-grpc_out=paths=source_relative:{output_dir}",
        str(proto_file)
    ]

    exit_code, stdout, stderr = run_command(cmd, cwd=root_dir)

    if exit_code != 0:
        console.print(f"[red]Error generating VectorXLite protos:[/red]")
        console.print(stderr)
        return False

    # Move generated files from nested directory to pb/
    nested_dir = output_dir / "vectorxlite" / "v1"
    if nested_dir.exists():
        for file in nested_dir.glob("*.go"):
            target = output_dir / file.name
            file.rename(target)
            console.print(f"  Moved: {file.name}")

        # Clean up empty directories
        try:
            (output_dir / "vectorxlite" / "v1").rmdir()
            (output_dir / "vectorxlite").rmdir()
        except OSError:
            pass

    console.print("[green]✓ VectorXLite protos generated successfully[/green]")

    # List generated files
    pb_files = list(output_dir.glob("*.pb.go"))
    if pb_files:
        console.print(f"\n  Generated {len(pb_files)} files in {output_dir}:")
        for f in pb_files:
            console.print(f"    - {f.name}")

    return True


def generate_cluster_protos(root_dir: Path) -> bool:
    """Generate Cluster protos (distributed cluster)"""
    console.print("\n[yellow]Generating Cluster protos...[/yellow]")

    proto_file = root_dir / "proto" / "cluster" / "v1" / "cluster.proto"
    output_dir = root_dir / "distributed" / "cluster" / "pkg" / "pb"

    if not proto_file.exists():
        console.print(f"[red]Error: Proto file not found: {proto_file}[/red]")
        return False

    output_dir.mkdir(parents=True, exist_ok=True)

    cmd = [
        "protoc",
        f"--proto_path={root_dir / 'proto'}",
        f"--go_out=paths=source_relative:{output_dir}",
        f"--go-grpc_out=paths=source_relative:{output_dir}",
        str(proto_file)
    ]

    exit_code, stdout, stderr = run_command(cmd, cwd=root_dir)

    if exit_code != 0:
        console.print(f"[red]Error generating Cluster protos:[/red]")
        console.print(stderr)
        return False

    # Move generated files from nested directory to pb/
    nested_dir = output_dir / "cluster" / "v1"
    if nested_dir.exists():
        for file in nested_dir.glob("*.go"):
            target = output_dir / file.name
            file.rename(target)
            console.print(f"  Moved: {file.name}")

        # Clean up empty directories
        try:
            (output_dir / "cluster" / "v1").rmdir()
            (output_dir / "cluster").rmdir()
        except OSError:
            pass

    console.print("[green]✓ Cluster protos generated successfully[/green]")

    # List generated files
    pb_files = list(output_dir.glob("*.pb.go"))
    if pb_files:
        console.print(f"\n  Generated {len(pb_files)} files in {output_dir}:")
        for f in pb_files:
            console.print(f"    - {f.name}")

    return True


def main():
    # Get the project root directory (parent of tools/, which is parent of tools/python/)
    root_dir = Path(__file__).parent.parent.parent.resolve()

    console.print(Panel.fit(
        "[bold cyan]VectorXLite Protocol Buffer Generator[/bold cyan]",
        subtitle=f"Root: {root_dir}"
    ))

    # Check if protoc is installed
    exit_code, _, _ = run_command(["protoc", "--version"])
    if exit_code != 0:
        console.print("[red]Error: protoc not found. Please install Protocol Buffer compiler.[/red]")
        sys.exit(1)

    success = True

    # Generate VectorXLite protos
    if not generate_vectorxlite_protos(root_dir):
        success = False

    # Generate Cluster protos
    if not generate_cluster_protos(root_dir):
        success = False

    if success:
        console.print("\n[bold green]✓ All proto files generated successfully![/bold green]")
        sys.exit(0)
    else:
        console.print("\n[bold red]✗ Some proto generation failed[/bold red]")
        sys.exit(1)


if __name__ == "__main__":
    main()
