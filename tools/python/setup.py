#!/usr/bin/env python3
"""
Setup script for VectorXLite tools

This script:
1. Creates a virtual environment
2. Installs dependencies
3. Verifies the installation

Usage:
    python tools/setup.py
"""

import subprocess
import sys
from pathlib import Path
from rich.console import Console

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


def main():
    tools_dir = Path(__file__).parent.parent.resolve()
    venv_dir = tools_dir / "venv"

    console.print("[bold cyan]VectorXLite Tools Setup[/bold cyan]\n")

    # Check if venv already exists
    if venv_dir.exists():
        console.print(f"[yellow]Virtual environment already exists at {venv_dir}[/yellow]")
        console.print("To recreate, delete the venv directory first.")
        sys.exit(0)

    # Create virtual environment
    console.print("[yellow]Creating virtual environment...[/yellow]")
    exit_code, stdout, stderr = run_command(
        [sys.executable, "-m", "venv", "venv"],
        cwd=tools_dir
    )

    if exit_code != 0:
        console.print(f"[red]Error creating venv: {stderr}[/red]")
        sys.exit(1)

    console.print("[green]✓ Virtual environment created[/green]")

    # Determine pip path
    if sys.platform == "win32":
        pip_path = venv_dir / "Scripts" / "pip"
    else:
        pip_path = venv_dir / "bin" / "pip"

    # Upgrade pip
    console.print("\n[yellow]Upgrading pip...[/yellow]")
    exit_code, stdout, stderr = run_command(
        [str(pip_path), "install", "--upgrade", "pip"],
        cwd=tools_dir
    )

    if exit_code != 0:
        console.print(f"[yellow]Warning: Could not upgrade pip: {stderr}[/yellow]")

    # Install requirements
    console.print("\n[yellow]Installing dependencies...[/yellow]")
    exit_code, stdout, stderr = run_command(
        [str(pip_path), "install", "-r", "requirements.txt"],
        cwd=tools_dir
    )

    if exit_code != 0:
        console.print(f"[red]Error installing dependencies: {stderr}[/red]")
        sys.exit(1)

    console.print("[green]✓ Dependencies installed[/green]")

    # Show installed packages
    console.print("\n[yellow]Installed packages:[/yellow]")
    exit_code, stdout, stderr = run_command(
        [str(pip_path), "list"],
        cwd=tools_dir
    )
    console.print(stdout)

    # Instructions
    console.print("\n[bold green]✓ Setup complete![/bold green]\n")
    console.print("To use the tools, activate the virtual environment:")

    if sys.platform == "win32":
        console.print("  [cyan]tools\\venv\\Scripts\\activate[/cyan]")
    else:
        console.print("  [cyan]source tools/venv/bin/activate[/cyan]")

    console.print("\nOr run tools directly:")
    console.print("  [cyan]tools/venv/bin/python tools/generate_protos.py[/cyan]")
    console.print("  [cyan]tools/venv/bin/python tools/start_cluster.py[/cyan]")
    console.print("  [cyan]tools/venv/bin/python tools/test_operations.py[/cyan]")


if __name__ == "__main__":
    main()
