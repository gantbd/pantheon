"""Test an explicit local Workbench source closure, without changing any checkout."""

import argparse
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile


ROOT = Path(__file__).resolve().parents[1]
PROJECTS = ("chronos", "chronos-tui", "vellum", "vellum-tui", "moira", "daimon")


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


def archive(repo, revision, output):
    output.mkdir()
    with tempfile.TemporaryFile() as bundle:
        subprocess.run(["git", "-C", str(repo), "archive", revision], stdout=bundle, check=True)
        bundle.seek(0)
        with tarfile.open(fileobj=bundle) as source:
            source.extractall(output, filter="data")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workbench", help="Workbench commit")
    parser.add_argument("--live", action="store_true", help="run the local socket and tmux gate")
    parser.add_argument(
        "--chronos", help="explicit Chronos candidate commit to test with Workbench"
    )
    parser.add_argument("--vellum", help="explicit Vellum candidate commit")
    parser.add_argument("--vellum-tui", help="explicit Vellum TUI candidate commit")
    args = parser.parse_args()

    workbench = git(ROOT, "rev-parse", "--verify", f"{args.workbench}^{{commit}}")
    receipt = json.loads(git(ROOT, "show", f"{workbench}:release/source.json"))
    if receipt.get("schema") != 2:
        raise SystemExit("Workbench candidate must contain a schema 2 source receipt")
    inputs = receipt["inputs"]
    if inputs.get("agoraTui", {}).get("revision") is None:
        raise SystemExit(
            "agora-tui has no immutable revision; pin the independent presentation peer before source qualification"
        )
    tui = inputs["chronosTui"]["revision"]
    tui_receipt = json.loads(
        git(ROOT.parent / "chronos-tui", "show", f"{tui}:release/source.json")
    )
    if tui_receipt.get("schema") != 1 or not tui_receipt.get("immutableSourceQualified"):
        raise SystemExit("Chronos TUI pin must contain its qualified local source receipt")
    pins = {
        "chronos": args.chronos or inputs["chronos"]["revision"],
        "chronos-tui": tui,
        "vellum": args.vellum or inputs["vellum"]["revision"],
        "vellum-tui": args.vellum_tui or inputs["vellumTui"]["revision"],
        **{peer: tui_receipt["inputs"][peer]["revision"] for peer in ("moira", "daimon")},
    }
    for name in PROJECTS:
        repo = ROOT.parent / name
        revision = pins[name]
        if not re.fullmatch(r"[0-9a-f]{40}", revision):
            raise SystemExit(f"{name} requires a full local commit ID")
        git(repo, "cat-file", "-e", f"{revision}^{{commit}}")

    with tempfile.TemporaryDirectory(prefix="workbench-source-", dir="/tmp") as tmp:
        stage = Path(tmp)
        archive(ROOT, workbench, stage / "workbench-tui")
        for name in PROJECTS:
            archive(ROOT.parent / name, pins[name], stage / name)
        directory = stage / "workbench-tui"
        subprocess.run(
            ["cargo", "test", "--all-targets", "--locked", "--offline"],
            cwd=directory,
            check=True,
        )
        if args.live:
            subprocess.run(["sh", "tests/live/dual_service.sh"], cwd=directory, check=True)
    print(json.dumps({"workbench-tui": workbench, **pins}, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
