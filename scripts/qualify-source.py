"""Test an explicit local Workbench source closure, without changing any checkout."""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parents[1]
PROJECTS = (
    "agora",
    "agora-tui",
    "chronos",
    "chronos-tui",
    "moira-tui",
    "daimon-tui",
    "hermes-tui",
    "aegis-tui",
    "grounding-tui",
    "vellum",
    "vellum-tui",
    "moira",
    "daimon",
    "hermes",
    "aegis",
    "dendrite",
    "keystone",
)


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


def archive(repo, revision, output):
    output.mkdir()
    with tempfile.TemporaryFile() as bundle:
        subprocess.run(["git", "-C", str(repo), "archive", revision], stdout=bundle, check=True)
        bundle.seek(0)
        with tarfile.open(fileobj=bundle) as source:
            source.extractall(output, filter="data")


def run_live_evidence(stage, environment):
    status = json.loads(
        (stage / "workbench-tui" / "composition-status.json").read_text()
    )
    evidence = []
    for peer in sorted(status["peers"]):
        for relative in status["peers"][peer]["evidence"]:
            if relative not in evidence:
                evidence.append(relative)
    for relative in evidence:
        if not re.fullmatch(r"[a-z0-9][a-z0-9-]*/[A-Za-z0-9_./-]+\.sh", relative):
            raise SystemExit(f"unsafe live evidence path: {relative}")
        script = stage / relative
        if not script.is_file() or stage not in script.resolve().parents:
            raise SystemExit(f"missing live evidence in source closure: {relative}")
        repository, script_relative = relative.split("/", 1)
        subprocess.run(
            ["sh", script_relative],
            cwd=stage / repository,
            env=environment,
            check=True,
        )


def prepare_live_dependency_closure(stage, environment):
    """Materialize lock-pinned non-Rust dependencies inside the archive."""
    environment = {**environment, "MIX_ENV": "test"}
    subprocess.run(
        ["mix", "deps.get", "--only", "test"],
        cwd=stage / "dendrite",
        env=environment,
        check=True,
    )


def prepare_cargo_source_closure(stage, directory, keystone_revision):
    """Seed the declared Keystone Git identity from its exact local object."""
    ambient_cargo_home = Path(
        os.environ.get("CARGO_HOME", Path.home() / ".cargo")
    ).resolve()
    cargo_home = stage / ".cargo-home"
    cargo_home.mkdir()
    registry = ambient_cargo_home / "registry"
    if not registry.is_dir():
        raise SystemExit("ambient Cargo registry closure is unavailable")
    (cargo_home / "registry").symlink_to(registry, target_is_directory=True)

    git_source = stage / ".source-git" / "keystone.git"
    git_source.parent.mkdir()
    subprocess.run(["git", "init", "--bare", str(git_source)], check=True)
    subprocess.run(
        [
            "git",
            "-C",
            str(git_source),
            "fetch",
            "--no-tags",
            str(ROOT.parent / "keystone"),
            f"{keystone_revision}:refs/commit/{keystone_revision}",
        ],
        check=True,
    )

    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("GIT_CONFIG_")
    }
    source_url = git_source.resolve().as_uri()
    environment.update(
        {
            "CARGO_HOME": str(cargo_home),
            "CARGO_NET_GIT_FETCH_WITH_CLI": "true",
            "GIT_CONFIG_COUNT": "1",
            "GIT_CONFIG_KEY_0": f"url.{source_url}.insteadOf",
            "GIT_CONFIG_VALUE_0": "https://github.com/gantbd/keystone",
        }
    )
    subprocess.run(
        ["cargo", "fetch", "--locked"],
        cwd=directory,
        env=environment,
        check=True,
    )
    environment["CARGO_NET_OFFLINE"] = "true"
    return environment


def validate_direct_path_closure(directory, stage):
    missing = []
    for relative_manifest in ("Cargo.toml", "cli/Cargo.toml"):
        manifest_path = directory / relative_manifest
        manifest = tomllib.loads(manifest_path.read_text())
        for dependency, declaration in manifest.get("dependencies", {}).items():
            if not isinstance(declaration, dict) or "path" not in declaration:
                continue
            target = (manifest_path.parent / declaration["path"]).resolve()
            if stage not in target.parents or not (target / "Cargo.toml").is_file():
                missing.append(
                    f"{dependency} ({declaration['path']} from {relative_manifest})"
                )
    if missing:
        raise SystemExit(
            "candidate source closure omits direct dependencies: " + "; ".join(missing)
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workbench", help="Workbench commit")
    parser.add_argument("--live", action="store_true", help="run the local socket and tmux gate")
    parser.add_argument(
        "--chronos", help="explicit Chronos candidate commit to test with Workbench"
    )
    parser.add_argument("--vellum", help="explicit Vellum candidate commit")
    parser.add_argument("--vellum-tui", help="explicit Vellum TUI candidate commit")
    parser.add_argument("--moira", help="explicit Moira candidate commit")
    parser.add_argument("--moira-tui", help="explicit Moira TUI candidate commit")
    parser.add_argument("--daimon", help="explicit Daimon candidate commit")
    parser.add_argument("--daimon-tui", help="explicit Daimon TUI candidate commit")
    parser.add_argument("--hermes", help="explicit Hermes candidate commit")
    parser.add_argument("--hermes-tui", help="explicit Hermes TUI candidate commit")
    parser.add_argument("--aegis", help="explicit Aegis candidate commit")
    parser.add_argument("--aegis-tui", help="explicit Aegis TUI candidate commit")
    parser.add_argument("--dendrite", help="explicit Dendrite candidate commit")
    parser.add_argument("--grounding-tui", help="explicit Grounding TUI candidate commit")
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
        "agora": inputs["agora"]["revision"],
        "agora-tui": inputs["agoraTui"]["revision"],
        "chronos": args.chronos or inputs["chronos"]["revision"],
        "chronos-tui": tui,
        "moira": args.moira or inputs["moira"]["revision"],
        "moira-tui": args.moira_tui or inputs["moiraTui"]["revision"],
        "daimon": args.daimon or inputs["daimon"]["revision"],
        "daimon-tui": args.daimon_tui or inputs["daimonTui"]["revision"],
        "hermes": args.hermes or inputs["hermes"]["revision"],
        "hermes-tui": args.hermes_tui or inputs["hermesTui"]["revision"],
        "aegis": args.aegis or inputs["aegis"]["revision"],
        "aegis-tui": args.aegis_tui or inputs["aegisTui"]["revision"],
        "dendrite": args.dendrite or inputs["dendrite"]["revision"],
        "keystone": inputs["keystone"]["revision"],
        "grounding-tui": args.grounding_tui or inputs["groundingTui"]["revision"],
        "vellum": args.vellum or inputs["vellum"]["revision"],
        "vellum-tui": args.vellum_tui or inputs["vellumTui"]["revision"],
    }
    for name in PROJECTS:
        repo = ROOT.parent / name
        revision = pins[name]
        if not re.fullmatch(r"[0-9a-f]{40}", revision):
            raise SystemExit(f"{name} requires a full local commit ID")
        git(repo, "cat-file", "-e", f"{revision}^{{commit}}")

    with tempfile.TemporaryDirectory(prefix="workbench-source-", dir="/tmp") as tmp:
        # macOS commonly exposes /tmp as a symlink to /private/tmp. Canonicalize
        # once so containment checks compare one path identity.
        stage = Path(tmp).resolve()
        archive(ROOT, workbench, stage / "workbench-tui")
        for name in PROJECTS:
            archive(ROOT.parent / name, pins[name], stage / name)
        directory = stage / "workbench-tui"
        validate_direct_path_closure(directory, stage)
        environment = prepare_cargo_source_closure(
            stage, directory, pins["keystone"]
        )
        subprocess.run(
            ["cargo", "test", "--all-targets", "--locked", "--offline"],
            cwd=directory,
            env=environment,
            check=True,
        )
        if args.live:
            prepare_live_dependency_closure(stage, environment)
            run_live_evidence(stage, environment)
    print(json.dumps({"workbench-tui": workbench, **pins}, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
