"""Test an explicit local Workbench source closure, without changing any checkout."""

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
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

ALLOWED_LICENSE_EXPRESSIONS = {
    "(MIT OR Apache-2.0) AND Unicode-3.0",
    "Apache-2.0",
    "Apache-2.0 OR BSL-1.0",
    "Apache-2.0 OR MIT",
    "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
    "Apache-2.0/MIT",
    "BSD-3-Clause",
    "MIT",
    "MIT OR Apache-2.0",
    "MIT OR Apache-2.0 OR LGPL-2.1-or-later",
    "MIT/Apache-2.0",
    "Unlicense OR MIT",
    "Zlib",
}


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


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_deterministic_tar_gz(source, destination):
    with destination.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w", format=tarfile.PAX_FORMAT) as bundle:
                for path in [source, *sorted(source.rglob("*"))]:
                    relative = path.relative_to(source.parent)
                    info = bundle.gettarinfo(str(path), arcname=str(relative))
                    info.uid = 0
                    info.gid = 0
                    info.uname = ""
                    info.gname = ""
                    info.mtime = 0
                    if path.is_dir():
                        info.mode = 0o755
                        bundle.addfile(info)
                    else:
                        info.mode = 0o755 if relative.parts[-2:] == ("bin", "workbench-tui") else 0o644
                        with path.open("rb") as payload:
                            bundle.addfile(info, payload)


def check_profile(binary, profile_path, environment, expected_success):
    result = subprocess.run(
        [
            str(binary),
            "headless",
            "--workspace-profile",
            str(profile_path),
            "--check-profile",
        ],
        env=environment,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if (result.returncode == 0) != expected_success:
        raise SystemExit(
            f"installed profile check returned {result.returncode}: {result.stdout}"
        )


def qualify_local_release(stage, directory, environment, receipt, artifact_output):
    release = receipt.get("localRelease", {})
    if release.get("qualified") is not True:
        raise SystemExit("candidate does not claim the selected local release gate")
    rustc = subprocess.check_output(["rustc", "-vV"], env=environment, text=True)
    host = next(
        (line.removeprefix("host: ") for line in rustc.splitlines() if line.startswith("host: ")),
        None,
    )
    if host != release.get("target"):
        raise SystemExit(f"local release target {host!r} is not selected by the candidate")
    qualified_host = f"{platform.system()} {platform.release()} {platform.machine()}"
    if qualified_host != release.get("qualifiedHost"):
        raise SystemExit(
            f"local release host {qualified_host!r} is not the qualified host"
        )

    release_environment = {
        **environment,
        "CARGO_TARGET_DIR": str(stage / ".release-target"),
    }
    subprocess.run(
        [
            "cargo",
            "build",
            "--release",
            "--package",
            "workbench-tui-cli",
            "--bin",
            "workbench-tui",
            "--locked",
            "--offline",
        ],
        cwd=directory,
        env=release_environment,
        check=True,
    )
    binary = stage / ".release-target" / "release" / "workbench-tui"
    if not binary.is_file():
        raise SystemExit("release build did not produce workbench-tui")

    metadata = json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"],
            cwd=directory,
            env=release_environment,
            text=True,
        )
    )
    inventory = set()
    for package in metadata["packages"]:
        license_expression = package.get("license")
        if license_expression not in ALLOWED_LICENSE_EXPRESSIONS:
            raise SystemExit(
                f"unreviewed license expression for {package['name']} {package['version']}: "
                f"{license_expression!r}"
            )
        inventory.add(
            (
                package["name"],
                package["version"],
                license_expression,
                package.get("source") or "local-source-closure",
            )
        )

    package_name = f"workbench-tui-0.1.0-{host}"
    package_root = stage / package_name
    (package_root / "bin").mkdir(parents=True)
    documentation = package_root / "share" / "doc" / "workbench-tui"
    documentation.mkdir(parents=True)
    configuration = package_root / "share" / "workbench-tui"
    configuration.mkdir(parents=True)
    shutil.copy2(binary, package_root / "bin" / "workbench-tui")
    for name in (
        "README.md",
        "CHANGELOG.md",
        "SECURITY.md",
        "LICENSE-APACHE",
        "LICENSE-MIT",
        "docs/local-release.md",
    ):
        target = documentation / Path(name).name
        shutil.copy2(directory / name, target)
    shutil.copy2(directory / "release" / "source.json", configuration / "source.json")
    shutil.copy2(
        directory / "config" / "workspace.example.json",
        configuration / "workspace.example.json",
    )
    inventory_path = documentation / "THIRD_PARTY_LICENSES.tsv"
    inventory_path.write_text(
        "name\tversion\tlicense\tsource\n"
        + "".join("\t".join(row) + "\n" for row in sorted(inventory))
    )

    checksums = []
    for path in sorted(package_root.rglob("*")):
        if path.is_file():
            checksums.append(f"{sha256(path)}  {path.relative_to(package_root)}\n")
    (package_root / "SHA256SUMS").write_text("".join(checksums))

    archive_path = stage / f"{package_name}.tar.gz"
    write_deterministic_tar_gz(package_root, archive_path)
    archive_digest = sha256(archive_path)

    install_root = stage / "installed"
    with tarfile.open(archive_path, "r:gz") as bundle:
        bundle.extractall(install_root, filter="data")
    installed = install_root / package_name
    installed_binary = installed / "bin" / "workbench-tui"
    profile_path = stage / "operator-profile.json"
    shutil.copy2(
        installed / "share" / "workbench-tui" / "workspace.example.json",
        profile_path,
    )
    profile_digest = sha256(profile_path)
    deployment = stage / "deployment"
    deployment.mkdir()
    replacement = deployment / ".workbench-tui.new"
    shutil.copy2(installed_binary, replacement)
    replacement.chmod(0o755)
    deployed_binary = deployment / "workbench-tui"
    os.replace(replacement, deployed_binary)
    check_profile(deployed_binary, profile_path, release_environment, True)
    shutil.copy2(installed_binary, replacement)
    replacement.chmod(0o755)
    os.replace(replacement, deployed_binary)
    check_profile(deployed_binary, profile_path, release_environment, True)
    if sha256(profile_path) != profile_digest:
        raise SystemExit("artifact replacement modified the operator-owned profile")

    unsupported = json.loads(profile_path.read_text())
    unsupported["version"] = release["profileSchema"] + 1
    unsupported_path = stage / "unsupported-profile.json"
    unsupported_path.write_text(json.dumps(unsupported))
    check_profile(deployed_binary, unsupported_path, release_environment, False)

    if artifact_output is not None:
        output = Path(artifact_output).resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(archive_path, output)
    return {
        "artifact": archive_path.name,
        "sha256": archive_digest,
        "target": host,
        "qualifiedHost": qualified_host,
        "packages": len(inventory),
        "profileSchema": release["profileSchema"],
        "replacement": "atomic-same-version-profile-preserved",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workbench", help="Workbench commit")
    parser.add_argument("--live", action="store_true", help="run the local socket and tmux gate")
    parser.add_argument(
        "--local-release",
        action="store_true",
        help="build and qualify the selected local binary artifact",
    )
    parser.add_argument(
        "--artifact",
        help="copy the qualified local release tarball to this path",
    )
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
    if args.artifact and not args.local_release:
        parser.error("--artifact requires --local-release")

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
    keystone_git = inputs["keystoneGit"]["revision"]
    for name in PROJECTS:
        repo = ROOT.parent / name
        revision = pins[name]
        if not re.fullmatch(r"[0-9a-f]{40}", revision):
            raise SystemExit(f"{name} requires a full local commit ID")
        git(repo, "cat-file", "-e", f"{revision}^{{commit}}")
    if not re.fullmatch(r"[0-9a-f]{40}", keystone_git):
        raise SystemExit("keystoneGit requires a full local commit ID")
    git(ROOT.parent / "keystone", "cat-file", "-e", f"{keystone_git}^{{commit}}")

    local_release = None
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
            stage, directory, keystone_git
        )
        subprocess.run(
            ["cargo", "test", "--all-targets", "--locked", "--offline"],
            cwd=directory,
            env=environment,
            check=True,
        )
        if args.local_release:
            local_release = qualify_local_release(
                stage, directory, environment, receipt, args.artifact
            )
        if args.live:
            prepare_live_dependency_closure(stage, environment)
            run_live_evidence(stage, environment)
    result = {"workbench-tui": workbench, **pins, "keystone-git": keystone_git}
    if local_release is not None:
        result["localRelease"] = local_release
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
