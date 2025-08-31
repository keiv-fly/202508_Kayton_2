#!/usr/bin/env python3
import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


def find_workspace_root(script_path: Path) -> Path:
    # crates/kayton_kernel/install_kernel.py → workspace root is parents[2]
    return script_path.resolve().parents[2]


def main() -> int:
    parser = argparse.ArgumentParser(description="Install the Kayton Jupyter kernel")
    parser.add_argument(
        "--profile",
        choices=["debug", "release"],
        default="debug",
        help="Build profile whose kernelspec to install (default: debug)",
    )
    parser.add_argument(
        "--name",
        default="kayton",
        help="Kernelspec name to register in Jupyter (default: kayton)",
    )
    parser.add_argument(
        "--target-dir",
        default=None,
        help="Override target directory (defaults to <workspace>/target)",
    )
    parser.add_argument(
        "--use-absolute-exe",
        action="store_true",
        help="Rewrite kernel.json argv[0] to the absolute path of the built kayton_kernel executable",
    )
    parser.add_argument(
        "--user",
        action="store_true",
        default=True,
        help="Install for current user (default: True)",
    )
    parser.add_argument(
        "--sys-prefix",
        action="store_true",
        help="Install into sys.prefix instead of user dir",
    )

    args = parser.parse_args()

    script_path = Path(__file__)
    workspace = find_workspace_root(script_path)
    target_dir = (
        Path(args.target_dir)
        if args.target_dir
        else workspace / "target"
    )
    spec_dir = target_dir / args.profile / "kayton_kernelspec" / "kayton"
    kernel_json_path = spec_dir / "kernel.json"

    if not kernel_json_path.exists():
        print(
            f"kernelspec not found at {kernel_json_path}. Build the kernel first: \n"
            f"  cargo build -p kayton_kernel --profile {args.profile}",
            file=sys.stderr,
        )
        return 1

    # Optionally rewrite argv[0] to absolute path of built binary
    if args.use_absolute_exe:
        exe_name = "kayton_kernel.exe" if os.name == "nt" else "kayton_kernel"
        exe_path = (target_dir / args.profile / exe_name).resolve()
        if not exe_path.exists():
            print(
                f"built executable not found at {exe_path}. Build first.",
                file=sys.stderr,
            )
            return 1
        try:
            with kernel_json_path.open("r", encoding="utf-8") as f:
                data = json.load(f)
            argv = data.get("argv", [])
            if argv:
                argv[0] = str(exe_path)
                data["argv"] = argv
                with kernel_json_path.open("w", encoding="utf-8") as f:
                    json.dump(data, f, indent=2)
                print(f"Rewrote argv[0] in {kernel_json_path} to {exe_path}")
        except Exception as e:
            print(f"warning: failed to rewrite kernel.json: {e}", file=sys.stderr)

    jupyter = shutil.which("jupyter")
    if not jupyter:
        print("jupyter executable not found on PATH. Please install Jupyter first.", file=sys.stderr)
        return 1

    cmd = [jupyter, "kernelspec", "install", str(spec_dir), "--name", args.name]
    if args.user and not args.sys_prefix:
        cmd.append("--user")
    if args.sys_prefix:
        cmd.append("--sys-prefix")

    print("Running:", " ".join(cmd))
    try:
        subprocess.check_call(cmd)
    except subprocess.CalledProcessError as e:
        print(f"kernelspec install failed with exit code {e.returncode}", file=sys.stderr)
        return e.returncode

    print(f"Installed Jupyter kernelspec '{args.name}' from {spec_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


