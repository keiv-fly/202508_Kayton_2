"""
Calculate line counts for all Rust (".rs") files in the repository.

Usage examples (from repo root or any directory):
  - Default (auto-detect repo root):
      python notebooks/calc_lines_in_files.py
  - Specify a root directory to scan:
      python notebooks/calc_lines_in_files.py --root .
  - Show only the summary (hide per-file details):
      python notebooks/calc_lines_in_files.py --no-details

By default, directories named "target" and ".git" are excluded.
"""

from pathlib import Path
import argparse
from typing import Iterable, List, Set, Tuple


def detect_repo_root() -> Path:
    """Return the repository root, assuming this file is under <root>/notebooks/.

    Falls back to the parent of this file's directory if the expected structure
    is not present.
    """
    this_file = Path(__file__).resolve()
    notebooks_dir = this_file.parent
    candidate_root = notebooks_dir.parent
    return candidate_root if candidate_root.exists() else this_file.parent


def should_exclude(path: Path, excluded_dirnames: Set[str]) -> bool:
    """Return True if any path component matches one of excluded_dirnames."""
    parts = set(path.parts)
    return any(name in parts for name in excluded_dirnames)


def find_rs_files(root: Path, excluded_dirnames: Set[str]) -> List[Path]:
    """Recursively find all .rs files under root, skipping excluded directories."""
    files: List[Path] = []
    for p in root.rglob("*.rs"):
        if should_exclude(p, excluded_dirnames):
            continue
        files.append(p)
    return files


def count_file_lines(file_path: Path) -> int:
    """Count logical lines in a text file, ignoring encoding errors."""
    try:
        with file_path.open("r", encoding="utf-8", errors="ignore") as f:
            # splitlines() counts the final line even if no trailing newline
            return len(f.read().splitlines())
    except OSError:
        return 0


def format_row(count: int, rel_path: Path) -> str:
    return f"{count:8d}  {rel_path.as_posix()}"


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Calculate lines in all .rs files in the repository."
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=detect_repo_root(),
        help="Root directory to scan (default: auto-detected repo root)",
    )
    parser.add_argument(
        "--exclude",
        action="append",
        default=["target", ".git"],
        help="Directory name to exclude (can be specified multiple times)",
    )
    parser.add_argument(
        "--no-details",
        action="store_true",
        help="Do not print per-file line counts; only show the summary",
    )

    args = parser.parse_args(list(argv) if argv is not None else None)

    root: Path = args.root.resolve()
    excluded: Set[str] = set(args.exclude or [])

    rs_files = find_rs_files(root, excluded)
    rs_files.sort(key=lambda p: p.relative_to(root).as_posix())

    per_file_counts: List[Tuple[Path, int]] = []
    total_lines = 0
    for fp in rs_files:
        count = count_file_lines(fp)
        total_lines += count
        per_file_counts.append((fp, count))

    if not args.no_details:
        print("Lines     File")
        print("--------  ----")
        for fp, count in per_file_counts:
            rel = fp.relative_to(root)
            print(format_row(count, rel))
        print()

    print(f"Total .rs files: {len(per_file_counts)}")
    print(f"Total lines: {total_lines}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())


