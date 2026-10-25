#!/usr/bin/env python3
"""Run the frozen order-balanced work-stealing campaign as fresh processes.

The runner builds the compare example from `--source` with `--build-command`
(default: the release cargo build), hashes the built binary, and identifies
the source tree it was built from, so a selection is attributed to a
verifiable build. Every invocation is retained as a row that starts as running and ends as ok,
failed or timeout. The output document is rewritten before and after each
invocation, so an interrupted campaign keeps its completed observations and
records the attempt that was in flight. Each repetition also runs the
global FIFO artifact twice under the labels A and B as an identical-artifact
control that flows through the same scheduling, parsing and analysis paths.
"""
import argparse
import hashlib
import json
import os
import platform
import re
import shlex
import shutil
import signal
import statistics
import subprocess
import tempfile
import time
from pathlib import Path

WORKLOADS = [
    {"name": "tiny_balanced", "workers": 2, "tasks": 256, "leaf_work": 0},
    {"name": "tiny_skewed", "workers": 2, "tasks": 256, "leaf_work": 0},
    {"name": "medium_balanced", "workers": 4, "tasks": 16384, "leaf_work": 8},
    {"name": "medium_skewed", "workers": 4, "tasks": 16384, "leaf_work": 8},
    {"name": "coarse_skewed", "workers": 4, "tasks": 4096, "leaf_work": 512},
]
BASELINE = "global_fifo"
CONTROL_LABELS = ["A", "B"]
BOUNDARY = 1.03
PROTOCOL_REPETITIONS = 8
TOPIC = Path(__file__).resolve().parent.parent
REPOSITORY = TOPIC.parent.parent
SOURCE_FILES = [
    "Cargo.lock",
    "Cargo.toml",
    "topics/028-work-stealing-deques/Cargo.toml",
    "topics/028-work-stealing-deques/examples/compare.rs",
    "topics/028-work-stealing-deques/scripts/run_campaign.py",
    "topics/028-work-stealing-deques/src/lib.rs",
    "topics/028-work-stealing-deques/tests/contracts.rs",
]


def candidates_for(workload):
    placement = "locked_steal_balanced" if workload["name"].endswith("balanced") else "locked_steal_skewed"
    return [BASELINE, placement]


def captured_text(output):
    """Decode captured child output with replacement so invalid UTF-8 is retained in the row instead of aborting the campaign."""
    if output is None:
        return ""
    if isinstance(output, bytes):
        output = output.decode("utf-8", errors="replace")
    return output.strip()


def schedule_for(repetition, candidates):
    """Four-repetition cycle over the slots (control, treatment, control, treatment) and its parity mirror.

    Over four repetitions each treatment candidate and each control label occupies every slot once, each
    candidate follows a control run three times and itself once, and the order within each pair rotates.
    """
    t0, t1 = [(candidate, candidate, "treatment") for candidate in candidates]
    c0, c1 = [(BASELINE, label, "control") for label in CONTROL_LABELS]
    cycle = [[c0, t0, c1, t1], [t1, c1, t0, c0], [c1, t1, c0, t0], [t0, c0, t1, c1]]
    return cycle[repetition % 4]


def start_row(binary, workload, candidate, label, arm, repetition=None):
    """Create the row for one invocation before launching it, so a killed attempt leaves a `running` record."""
    command = [
        str(binary), "--candidate", candidate, "--workers", str(workload["workers"]),
        "--tasks", str(workload["tasks"]), "--leaf-work", str(workload["leaf_work"]),
    ]
    row = {"candidate": candidate, "command": command, "workload": workload, "label": label, "arm": arm, "status": "running", "started_unix_s": time.time()}
    if repetition is not None:
        row["repetition"] = repetition
    return row


def run_row(row, workload, timeout):
    """Execute the row's command and replace `running` with ok, failed or timeout."""
    try:
        process = subprocess.run(row["command"], capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired as expired:
        row.update({"status": "timeout", "timeout_s": timeout, "stdout": captured_text(expired.stdout), "stderr": captured_text(expired.stderr)})
        return row
    except OSError as error:
        row.update({"status": "failed", "reason": f"launch failed: {error}"})
        return row
    stdout = captured_text(process.stdout)
    row.update({"stdout": stdout, "stderr": captured_text(process.stderr), "returncode": process.returncode})
    fields = dict(item.split("=", 1) for item in stdout.split() if "=" in item)
    expected = {"candidate": row["candidate"], "workers": str(workload["workers"]), "tasks": str(workload["tasks"]), "leaf_work": str(workload["leaf_work"])}
    if process.returncode != 0:
        row.update({"status": "failed", "reason": f"exit code {process.returncode}"})
    elif any(fields.get(key) != value for key, value in expected.items()):
        row.update({"status": "failed", "reason": f"echoed arguments {fields} differ from {expected}"})
    elif not fields.get("elapsed_ns", "").isdigit() or int(fields["elapsed_ns"]) <= 0:
        row.update({"status": "failed", "reason": f"elapsed_ns {fields.get('elapsed_ns')!r} is not a positive integer"})
    else:
        row.update({"status": "ok", "elapsed_ns": int(fields["elapsed_ns"])})
    return row


def probe(command, cwd=None, env=None):
    try:
        return subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True, check=True, timeout=60).stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return None


def sha256_of(path):
    """SHA-256 of a regular file; None when the path is missing or cannot be read, so validation treats it as unavailable."""
    try:
        return hashlib.sha256(Path(path).read_bytes()).hexdigest() if Path(path).is_file() else None
    except OSError:
        return None


CPU_IDENTITY_FIELDS = ("model name", "hardware", "cpu implementer", "cpu variant", "cpu part", "cpu revision")


def cpu_identification(cpuinfo=Path("/proc/cpuinfo")):
    """The first processor block of cpuinfo, in the form the evidence receipt stores."""
    return cpuinfo.read_text(errors="replace").split("\n\n", 1)[0] if cpuinfo.is_file() else None


def cpu_model(cpuinfo=Path("/proc/cpuinfo")):
    """Identify the CPU from the first processor block: the model name on x86, implementer/variant/part/revision on Arm."""
    if not cpuinfo.is_file():
        return platform.processor() or None
    block = cpuinfo.read_text(errors="replace").split("\n\n", 1)[0]
    fields = {}
    for line in block.splitlines():
        key, _, value = line.partition(":")
        if key.strip().lower() in CPU_IDENTITY_FIELDS:
            fields[key.strip()] = value.strip()
    if "model name" in fields:
        return fields["model name"]
    return ", ".join(f"{key} {value}" for key, value in fields.items()) or platform.processor() or None


def worktree_status(repository, out, target=None):
    """Return porcelain status lines for tracked changes, untracked files and ignored Cargo inputs.

    Untracked files count because Cargo reads inputs such as `build.rs` or `.cargo/config.toml` without
    them being committed, and an ignored file matching a Cargo input pattern counts for the same reason.
    The campaign output and the target directory are generated and are left out. Returns None when git
    is unavailable.
    """
    status = probe(["git", "status", "--porcelain", "--untracked-files=all", "--ignored=matching"], cwd=repository)
    if status is None:
        return None
    root = Path(repository).resolve()
    generated = []
    for path in (out, target):
        try:
            generated.append(str(Path(path).resolve().relative_to(root)) if path is not None else None)
        except ValueError:
            generated.append(None)
    def is_generated(path):
        return path in generated or any(g is not None and path.startswith(g + "/") for g in generated)

    lines = []
    for line in status.splitlines():
        path = line[3:]
        if is_generated(path):
            continue
        if line.startswith("!!") and path.endswith("/"):
            listing = probe(["git", "ls-files", "--others", "--ignored", "--exclude-standard", "--", path], cwd=repository) or ""
            lines.extend(f"!! {inner}" for inner in listing.splitlines() if re.fullmatch(CARGO_INPUT_PATTERN, inner) and not is_generated(inner))
            continue
        if line.startswith("!!") and not re.fullmatch(CARGO_INPUT_PATTERN, path):
            continue
        lines.append(line)
    return lines


def toolchain_command(toolchain, command):
    """Route a compiler probe through rustup so it reports the toolchain that built the binary, not the directory override."""
    return ["rustup", "run", toolchain] + command if toolchain else command


def pin_binary(binary, staging):
    """Copy the binary into a private directory so every invocation runs the artifact that was hashed.

    Returns the copy, or None when the copy cannot be made; the campaign then runs the original path
    so launch failures are retained, and the document is marked invalid without a selection.
    """
    try:
        return Path(shutil.copy2(binary, staging / Path(binary).name))
    except OSError:
        return None


RUSTFLAGS_SOURCES = ("CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "RUSTFLAGS")
COMPILER_OVERRIDES = ("RUSTC", "CARGO_BUILD_RUSTC", "RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER")
CARGO_INPUT_PATTERN = r".*(\.rs|Cargo\.toml|Cargo\.lock|\.cargo/config(\.toml)?|rust-toolchain(\.toml)?)$"
RUNTIME_ENV_PATTERN = r"(LD_|DYLD_|GLIBC_|MALLOC_|Malloc|RUST_|RUSTUP_|OMP_|GOMP_|TCMALLOC_|JEMALLOC_|MIMALLOC_|MALLOC)[A-Za-z0-9_]*"


def compiler_overrides(source):
    """Cargo's compiler override variables with a hash of each named executable, so a wrapper is part of the identity.

    Cargo resolves a relative override against the build's working directory, so paths resolve against `source`.
    """
    result = {}
    for name in COMPILER_OVERRIDES:
        value = os.environ.get(name)
        if value is None:
            result[name] = None
            continue
        if value == "" and "WRAPPER" in name:
            result[name] = {"value": "", "disabled": True}
            continue
        if os.sep in value:
            resolved = str((Path(source) / value).resolve())
        else:
            resolved = which_from(value, source) or value
        result[name] = {"value": value, "resolved": resolved, "sha256": sha256_of(resolved)}
    return result


def cargo_profile():
    """Profile overrides Cargo reads from the environment, which change the generated release executable."""
    return {name: value for name, value in sorted(os.environ.items()) if name.startswith("CARGO_PROFILE_")}


def runtime_env():
    """Environment variables inherited by every measured child that can change allocator, loader or thread behaviour;
    a name outside the known knobs is kept only as a hash."""
    return {name: redacted_value(name, value) for name, value in sorted(os.environ.items()) if re.fullmatch(RUNTIME_ENV_PATTERN, name)}


def target_spec_sha256(source):
    """Hash of a custom target specification named by CARGO_BUILD_TARGET; None for a builtin triple or an unreadable file."""
    target = os.environ.get("CARGO_BUILD_TARGET")
    if not target or not target.endswith(".json"):
        return None
    return sha256_of((Path(source) / target).resolve())


EXTERNAL_INPUT_FLAGS = ("profile-use=", "profile-sample-use=", "instrument-coverage=", "link-arg=@", "link-args=@")


def rustflags_external_inputs(source, extra_tokens=()):
    """Files that compiler flags name as inputs (PGO profiles, response files), hashed; a missing file hashes to None."""
    flags = []
    for name in RUSTFLAGS_SOURCES + tuple(k for k in os.environ if re.fullmatch(r"CARGO_TARGET_[A-Z0-9_]+_RUSTFLAGS", k)):
        value = os.environ.get(name)
        if value:
            flags.extend(value.split("\x1f") if name == "CARGO_ENCODED_RUSTFLAGS" else shlex.split(value))
    flags.extend(extra_tokens)
    inputs = {}
    for token in flags:
        body = token[2:] if token.startswith("-C") and not token.startswith("-C ") else token
        for prefix in EXTERNAL_INPUT_FLAGS:
            if body.startswith(prefix):
                path = body[len(prefix):]
                if path:
                    resolved = (Path(source) / path).resolve()
                    inputs[str(resolved)] = sha256_of(resolved)
    return inputs


def rustflags_identity(source):
    """Record every Cargo flag source, plus hashes of the config files Cargo reads: `.cargo/config{,.toml}` in the
    build directory and each ancestor, then Cargo home."""
    cargo_home = (Path(source) / os.environ["CARGO_HOME"]).resolve() if os.environ.get("CARGO_HOME") else Path.home() / ".cargo"
    directories = [Path(source).resolve(), *Path(source).resolve().parents, cargo_home]
    candidates = [directory / ".cargo" / name for directory in directories[:-1] for name in ("config.toml", "config")]
    candidates += [cargo_home / "config.toml", cargo_home / "config"]
    return {
        **{name: os.environ.get(name) for name in RUSTFLAGS_SOURCES},
        "target_specific": {name: value for name, value in os.environ.items() if re.fullmatch(r"CARGO_TARGET_[A-Z0-9_]+_RUSTFLAGS", name)},
        "CARGO_BUILD_TARGET": os.environ.get("CARGO_BUILD_TARGET"),
        "target_spec_sha256": target_spec_sha256(source),
        "external_inputs": rustflags_external_inputs(source),
        "cargo_config_sha256": {str(path): sha256_of(path) for path in candidates if path.is_file()},
    }


def identity(source, binary, pinned, out, toolchain=None, target=None, chain=None, invocation_env=None):
    status = worktree_status(source, out, target)
    probe_env = dict(os.environ, **(invocation_env or {}))
    compilers = [member["resolved"] for member in (chain or []) if member["role"] == "compiler"]
    cargos = [member["resolved"] for member in (chain or []) if member["role"] == "cargo"]
    linkers = [member["resolved"] for member in (chain or []) if member["role"] in ("linker", "target-linker")]
    rustc = compilers[-1] if compilers else (os.environ.get("RUSTC") or os.environ.get("CARGO_BUILD_RUSTC") or "rustc")
    cargo = cargos[-1] if cargos else "cargo"
    return {
        "source": str(source),
        "hostname": platform.node(),
        "platform": platform.platform(),
        "uname": list(platform.uname()),
        "uname_a": probe(["uname", "-a"]),
        "python": platform.python_version(),
        "cpu_model": cpu_model(),
        "cpu_identification": cpu_identification(),
        "available_cpus": len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else os.cpu_count(),
        "affinity": sorted(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None,
        "source_commit": probe(["git", "rev-parse", "HEAD"], cwd=source),
        "worktree_dirty": None if status is None else bool(status),
        "worktree_status": status,
        "source_sha256": {name: sha256_of(Path(source) / name) for name in SOURCE_FILES},
        "runner": str(Path(__file__).resolve()),
        "runner_sha256": sha256_of(Path(__file__).resolve()),
        "runner_matches_source": sha256_of(Path(__file__).resolve()) == sha256_of(Path(source) / "topics/028-work-stealing-deques/scripts/run_campaign.py"),
        "binary": str(binary),
        "binary_copy": None if pinned is None else str(pinned),
        "binary_sha256": None if pinned is None else sha256_of(pinned),
        "toolchain": toolchain,
        "rustc": probe(toolchain_command(None if chain else toolchain, [rustc, "-Vv"]), cwd=source, env=probe_env),
        "cargo": probe(toolchain_command(None if cargos else toolchain, [cargo, "-V"]), cwd=source, env=probe_env),
        "linker": probe([linkers[-1], "--version"], cwd=source, env=probe_env) if linkers else None,
        "compiler_overrides": compiler_overrides(source),
        "cargo_profile": cargo_profile(),
        "runtime_env": runtime_env(),
        "rustflags": rustflags_identity(source),
    }


def default_build_command(toolchain):
    """Clean the topic package first so Cargo recompiles it and prints the verbose invocation the identity is read from."""
    plus = f" +{toolchain}" if toolchain else ""
    return (f"cargo{plus} clean --release -p topic028-work-stealing-deques && "
            f"cargo{plus} build --release -p topic028-work-stealing-deques --example compare -vv --message-format=json-render-diagnostics")


PROVENANCE_ENV_NAMES = {
    "CARGO", "CARGO_CRATE_NAME", "CARGO_BIN_NAME", "CARGO_MANIFEST_DIR", "CARGO_MANIFEST_PATH", "CARGO_PRIMARY_PACKAGE",
    "CARGO_SBOM_PATH", "CARGO_TARGET_TMPDIR", "CARGO_HOME", "CARGO_TARGET_DIR", "CARGO_BUILD_TARGET", "CARGO_BUILD_TARGET_DIR",
    "CARGO_BUILD_RUSTC", "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS", "RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTDOCFLAGS",
    "RUST_TARGET_PATH", "RUST_BACKTRACE", "RUST_MIN_STACK", "RUST_LOG", "RUSTUP_TOOLCHAIN", "RUSTUP_HOME",
    "PATH", "LD_LIBRARY_PATH", "LD_PRELOAD", "LD_BIND_NOW", "DYLD_LIBRARY_PATH", "DYLD_FALLBACK_LIBRARY_PATH", "DYLD_INSERT_LIBRARIES",
    "GLIBC_TUNABLES", "MALLOC_ARENA_MAX", "MALLOC_ARENA_TEST", "MALLOC_TRIM_THRESHOLD_", "MALLOC_MMAP_THRESHOLD_",
    "MALLOC_MMAP_MAX_", "MALLOC_TOP_PAD_", "MALLOC_PERTURB_", "MALLOC_CHECK_", "MallocNanoZone", "MallocStackLogging",
    "MallocScribble", "MallocGuardEdges", "OMP_NUM_THREADS", "OMP_PROC_BIND", "OMP_PLACES", "GOMP_CPU_AFFINITY",
}
PROVENANCE_ENV_PATTERNS = (
    re.compile(r"CARGO_PKG_[A-Z_]+"),
    re.compile(r"CARGO_PROFILE_[A-Z_]+_(OPT_LEVEL|LTO|CODEGEN_UNITS|DEBUG|DEBUG_ASSERTIONS|OVERFLOW_CHECKS|PANIC|INCREMENTAL|RPATH|STRIP|SPLIT_DEBUGINFO)"),
    re.compile(r"CARGO_TARGET_[A-Z0-9_]+_(RUSTFLAGS|LINKER|RUNNER|AR)"),
    re.compile(r"(TCMALLOC|JEMALLOC|MIMALLOC)_[A-Z_]+"),
)


def is_provenance_variable(key):
    """Only exact, known build and runtime knobs are recorded verbatim; everything else is kept as a hash."""
    return key in PROVENANCE_ENV_NAMES or any(pattern.fullmatch(key) for pattern in PROVENANCE_ENV_PATTERNS)


def redacted_value(key, value):
    """Keep provenance variables verbatim; anything else (an `[env]` credential, for example) is kept only as a hash."""
    if is_provenance_variable(key):
        return value
    return {"sha256": hashlib.sha256(value.encode()).hexdigest()}


def redact_running_lines(text):
    """Replace the values of non-provenance environment prefixes in Cargo's verbose `Running` lines."""
    def redact(match):
        tokens = shlex.split(match.group(1))
        out = []
        for token in tokens:
            key, sep, value = token.partition("=")
            if sep and not token.startswith(("/", ".")) and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", key) and not is_provenance_variable(key):
                out.append(f"{key}=<redacted>")
            else:
                out.append(shlex.quote(token) if " " in token else token)
        return "Running `" + " ".join(out) + "`"
    return re.sub(r"Running `(.+)`", redact, text)


def compiler_invocations(stderr, source):
    """Every verbose `Running` line Cargo printed for the compare example, parsed into
    (chain, target, invocation environment); chain members are {role, path, resolved, sha256}."""
    results = []
    for line in stderr.splitlines():
        match = re.search(r"Running `(.+)`\s*$", line)
        if not match or "--crate-name compare" not in match.group(1):
            continue
        tokens = shlex.split(match.group(1))
        executables, invocation_env = [], {}
        for token in tokens:
            if token.startswith("--") or token.startswith("-C"):
                break
            if "=" in token and not token.startswith(("/", ".")):
                key, _, value = token.partition("=")
                invocation_env[key] = value
                continue
            executables.append(token)
        target = tokens[tokens.index("--target") + 1] if "--target" in tokens and tokens.index("--target") + 1 < len(tokens) else None
        linker = None
        for index, token in enumerate(tokens):
            if token.startswith("-Clinker="):
                linker = token[len("-Clinker="):]
            elif token == "-C" and index + 1 < len(tokens) and tokens[index + 1].startswith("linker="):
                linker = tokens[index + 1][len("linker="):]
        roles = ["wrapper"] * (len(executables) - 1) + ["compiler"] if executables else []
        members = list(zip(roles, executables)) + ([("linker", linker)] if linker else [])
        chain = []
        if invocation_env.get("CARGO"):
            members.append(("cargo", invocation_env["CARGO"]))
        search_path = invocation_env.get("PATH")
        for role, path in members:
            resolved = str((Path(source) / path).resolve()) if os.sep in path else (which_from(path, source, search_path) or path)
            chain.append({"role": role, "path": path, "resolved": resolved, "sha256": sha256_of(resolved)})
        recorded_env = {key: redacted_value(key, value) for key, value in invocation_env.items()}
        results.append((chain, target, invocation_env, recorded_env))
    return results


def target_spec_path(target, source, invocation_env=None):
    """The custom target specification file behind `target`: a `.json` path, or `<name>.json` found via the
    RUST_TARGET_PATH the compiler saw (Cargo's invocation environment first, then the runner's)."""
    if not target:
        return None
    if target.endswith(".json"):
        return (Path(source) / target).resolve()
    search = (invocation_env or {}).get("RUST_TARGET_PATH", os.environ.get("RUST_TARGET_PATH", ""))
    for directory in filter(None, search.split(os.pathsep)):
        candidate = (Path(source) / directory).resolve() / f"{target}.json"
        if candidate.is_file():
            return candidate
    return None


def effective_target_spec(target, source, invocation_env=None):
    """Hash of the custom target specification behind `target`, or None when there is none or it cannot be read."""
    path = target_spec_path(target, source, invocation_env)
    return None if path is None else sha256_of(path)


def target_spec_linker(target, source, invocation_env=None):
    """The linker a custom target specification names in its `linker` field, resolved like a command-line tool."""
    path = target_spec_path(target, source, invocation_env)
    if path is None or not path.is_file():
        return None
    try:
        linker = json.loads(path.read_text()).get("linker")
    except (OSError, ValueError, AttributeError):
        return None
    if not linker:
        return None
    resolved = str((Path(source) / linker).resolve()) if os.sep in linker else (which_from(linker, source, (invocation_env or {}).get("PATH")) or linker)
    return {"role": "target-linker", "path": linker, "resolved": resolved, "sha256": sha256_of(resolved)}


def target_is_builtin(target, chain, source):
    """Whether `target` is one of the compiler's builtin targets; None when the compiler cannot be asked."""
    compilers = [member["resolved"] for member in (chain or []) if member["role"] == "compiler"]
    if not target or target.endswith(".json") or not compilers:
        return None
    listing = probe([compilers[-1], "--print", "target-list"], cwd=source)
    return None if listing is None else target in listing.split()


def built_executables(stdout):
    """Every path Cargo reports for the compare example in its JSON compiler-artifact messages."""
    found = []
    for line in stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if isinstance(message, dict) and message.get("reason") == "compiler-artifact":
            target = message.get("target") or {}
            if target.get("name") == "compare" and "example" in (target.get("kind") or []) and message.get("executable"):
                found.append(Path(message["executable"]))
    return found


def which_from(name, source, path=None):
    """Locate a bare tool name on the given PATH (default: the runner's), resolving relative entries against the
    build directory as the build does."""
    for entry in (os.environ.get("PATH", "") if path is None else path).split(os.pathsep):
        candidate = (Path(source) / (entry or ".")).resolve() / name
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)
    return None


def build(source, command, timeout):
    """Run the build command in the source tree, in its own process group, and record its outcome.

    On timeout the whole group is killed so compiler descendants cannot outlive the runner.
    """
    record = {"command": command, "cwd": str(source), "started_unix_s": time.time()}
    try:
        process = subprocess.Popen(command, shell=True, cwd=source, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    except OSError as error:
        record.update({"status": "failed", "reason": f"launch failed: {error}"})
        return record
    try:
        stdout, stderr = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        stdout, stderr = process.communicate()
        record.update({"status": "timeout", "timeout_s": timeout, "stdout": captured_text(stdout)[-4000:], "stderr": captured_text(stderr)[-4000:]})
        return record
    text = captured_text(stdout)
    errors = captured_text(stderr)
    executables = built_executables(text)
    invocations = compiler_invocations(errors, source)
    record["raw_invocation_env"] = None
    chain, target, invocation_env, recorded_env = invocations[0] if len(invocations) == 1 else (None, None, None, None)
    spec_linker = target_spec_linker(target, source, invocation_env)
    if chain is not None and spec_linker is not None:
        chain = chain + [spec_linker]
    if chain is not None and not any(member["role"] in ("linker", "target-linker") for member in chain):
        inferred = "link.exe" if (target or "").endswith("msvc") else "cc"
        resolved = which_from(inferred, source, (invocation_env or {}).get("PATH")) or inferred
        chain = chain + [{"role": "linker", "inferred": True, "path": inferred, "resolved": resolved, "sha256": sha256_of(resolved)}]
    record.update({
        "returncode": process.returncode, "stdout": redact_running_lines(text)[-4000:], "stderr": redact_running_lines(errors)[-4000:],
        "executable": str(executables[0]) if len(executables) == 1 else None,
        "reported_executables": [str(path) for path in executables],
        "compiler_invocations": len(invocations),
        "compiler_chain": chain, "invocation_env": recorded_env, "target": target,
        "target_spec_sha256": effective_target_spec(target, source, invocation_env),
        "target_is_builtin": target_is_builtin(target, chain, source),
    })
    record["raw_invocation_env"] = invocation_env
    if process.returncode != 0:
        record["status"] = "failed"
    elif len(executables) > 1 or len(invocations) > 1:
        record["status"] = "ambiguous"
        record["reason"] = f"{len(executables)} reported artifacts, {len(invocations)} compiler invocations for the compare example"
    else:
        record["status"] = "ok"
    return record


def target_directory(source, toolchain):
    """Cargo's effective target directory for the source tree, honouring CARGO_TARGET_DIR and build.target-dir."""
    metadata = probe(toolchain_command(toolchain, ["cargo", "metadata", "--no-deps", "--format-version", "1"]), cwd=source)
    try:
        return Path(json.loads(metadata)["target_directory"]) if metadata else None
    except (ValueError, KeyError, TypeError):
        return None


def paired_ratios(rows, numerator, denominator, repetitions):
    """Return numerator/denominator per repetition and the count of repetitions missing an ok pair."""
    by_label = {}
    for row in rows:
        if row["status"] == "ok":
            by_label.setdefault(row["label"], {})[row["repetition"]] = row["elapsed_ns"]
    ratios, incomplete = [], 0
    for repetition in range(repetitions):
        top = by_label.get(numerator, {}).get(repetition)
        bottom = by_label.get(denominator, {}).get(repetition)
        if top is None or bottom is None:
            incomplete += 1
        else:
            ratios.append(top / bottom)
    return ratios, incomplete


def summarize(rows, warmups, repetitions):
    summaries = []
    for workload in WORKLOADS:
        cell = [r for r in rows if r["workload"]["name"] == workload["name"]]
        warmup_failures = len([w for w in warmups if w["workload"]["name"] == workload["name"] and w["status"] != "ok"])
        baseline, challenger = candidates_for(workload)
        observed = {}
        for label in [baseline, challenger] + CONTROL_LABELS:
            values = [r["elapsed_ns"] for r in cell if r["label"] == label and r["status"] == "ok"]
            observed[label] = {"median_ns": statistics.median(values) if values else None, "min_ns": min(values, default=None), "max_ns": max(values, default=None), "samples": len(values)}
        ratios, incomplete = paired_ratios(cell, challenger, baseline, repetitions)
        aa_ratios, aa_incomplete = paired_ratios(cell, CONTROL_LABELS[0], CONTROL_LABELS[1], repetitions)
        if aa_incomplete:
            aa_null_check = "incomplete"
        elif all(r < 1 for r in aa_ratios) or all(r > 1 for r in aa_ratios):
            aa_null_check = "failed"
        else:
            aa_null_check = "passed"
        if incomplete or warmup_failures or aa_null_check != "passed" or repetitions != PROTOCOL_REPETITIONS:
            selected = "unresolved"
        elif all(r < 1 / BOUNDARY for r in ratios):
            selected = challenger
        elif all(r > BOUNDARY for r in ratios):
            selected = baseline
        else:
            selected = "unresolved"
        ranked = [label for label in (baseline, challenger) if observed[label]["median_ns"] is not None]
        summaries.append({
            "workload": workload,
            "candidates": observed,
            "lowest_median": min(ranked, key=lambda label: observed[label]["median_ns"]) if ranked else None,
            "paired_ratios": ratios,
            "ratio_range": [min(ratios), max(ratios)] if ratios else None,
            "incomplete_repetitions": incomplete,
            "selected": selected,
            "aa_ratios": aa_ratios,
            "aa_incomplete_repetitions": aa_incomplete,
            "aa_null_check": aa_null_check,
            "failures": len([r for r in cell if r["status"] != "ok"]),
            "warmup_failures": warmup_failures,
        })
    return summaries


def write_document(out, document):
    out.parent.mkdir(parents=True, exist_ok=True)
    scratch = out.with_name(out.name + ".partial")
    scratch.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    os.replace(scratch, out)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, default=REPOSITORY, help="checkout to build and identify (default: this runner's repository)")
    parser.add_argument("--binary", type=Path, help="built example (default: <cargo target directory>/release/examples/compare)")
    parser.add_argument("--build-command", help="shell command run in --source that produces --binary (default: the release cargo build)")
    parser.add_argument("--build-timeout", type=float, default=1800.0, help="seconds allowed for the build")
    parser.add_argument("--skip-build", action="store_true", help="use the existing --binary; its provenance is then unverified and no selection is emitted")
    parser.add_argument("--repetitions", type=int, default=PROTOCOL_REPETITIONS, help="paired repetitions per cell; a selection is emitted only for the frozen protocol's eight")
    parser.add_argument("--timeout", type=float, default=60.0, help="seconds allowed per process")
    parser.add_argument("--toolchain", help="rustup toolchain that built --binary, e.g. 1.98.1; compiler probes run through it")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.repetitions < 4 or args.repetitions % 2:
        raise SystemExit("--repetitions must be even and at least four")
    source = args.source.resolve()
    lexical_out = Path(os.path.abspath(args.out))
    if lexical_out.is_symlink() or probe(["git", "ls-files", "--error-unmatch", "--", str(lexical_out)], cwd=source):
        raise SystemExit(f"--out {args.out} is a symlink or names a file tracked in {source}; choose a generated path")
    if re.fullmatch(CARGO_INPUT_PATTERN, str(lexical_out)):
        raise SystemExit(f"--out {args.out} matches a Cargo input pattern; choose a generated path")
    target = target_directory(source, args.toolchain)
    if target is not None:
        try:
            inside = target.resolve().relative_to(source)
        except ValueError:
            inside = None
        if inside is not None and probe(["git", "ls-files", "--", str(inside)], cwd=source):
            raise SystemExit(f"Cargo's target directory {target} contains tracked inputs of {source}; the build would hide source changes")
    if args.binary is not None:
        built = args.binary if args.binary.is_absolute() else source / args.binary
    else:
        built = (target or source / "target") / "release" / "examples" / "compare"
    before = sha256_of(built) if args.binary is not None else None
    if args.skip_build:
        build_record = {"status": "skipped", "command": None, "cwd": str(source), "executable": None}
    else:
        build_record = build(source, args.build_command or default_build_command(args.toolchain), args.build_timeout)
        reported = source / build_record["executable"] if build_record.get("executable") else None
        if args.binary is None and reported is not None:
            built = reported
        elif args.binary is None and build_record["status"] == "ok":
            build_record["status"] = "no artifact"
        elif args.binary is not None and build_record["status"] == "ok":
            if reported is not None and reported.resolve() != built.resolve():
                build_record["status"] = "artifact mismatch"
                build_record["reason"] = f"build reported {reported}, --binary names {built}"
            elif reported is None and (not built.is_file() or sha256_of(built) == before):
                build_record["status"] = "artifact mismatch"
                build_record["reason"] = f"{built} is unchanged by this build" if built.is_file() else f"{built} was not produced by this build"
    build_record["target_directory"] = None if target is None else str(target)
    staging = Path(tempfile.mkdtemp(prefix="topic028-campaign-"))
    pinned = pin_binary(built, staging) if build_record["status"] in ("ok", "skipped") else None
    binary = built if pinned is None else pinned
    document = {
        "status": "running",
        "boundary": "scheduler lifecycle: queues, threads, task work, counters, joins and equality; sequential oracle before timer",
        "selection_rule": "lowest median is descriptive; select only when every paired steal/global ratio passes the same 3 percent boundary; otherwise unresolved",
        "control": "identical global_fifo artifact labelled A and B once each per repetition",
        "order": "four-repetition cycle: [A, base, B, chal], [chal, B, base, A], [B, chal, A, base], [base, A, chal, B]; every label occupies each slot once per cycle and each treatment follows a control run three times and itself once",
        "repetitions": args.repetitions,
        "protocol_repetitions": args.repetitions == PROTOCOL_REPETITIONS,
        "timeout_s": args.timeout,
        "build": build_record,
        "environment": identity(source, built, pinned, args.out, args.toolchain, target, build_record.get("compiler_chain"), build_record.pop("raw_invocation_env", None)),
        "created_unix_s": time.time(),
        "warmups": [],
        "rows": [],
        "summaries": [],
    }
    write_document(args.out, document)
    for workload in WORKLOADS if build_record["status"] in ("ok", "skipped") else []:
        candidates = candidates_for(workload)
        for candidate in candidates:
            row = start_row(binary, workload, candidate, candidate, "warmup")
            document["warmups"].append(row)
            write_document(args.out, document)
            run_row(row, workload, args.timeout)
            write_document(args.out, document)
        for repetition in range(args.repetitions):
            for candidate, label, arm in schedule_for(repetition, candidates):
                row = start_row(binary, workload, candidate, label, arm, repetition)
                document["rows"].append(row)
                write_document(args.out, document)
                run_row(row, workload, args.timeout)
                write_document(args.out, document)
    document["environment"]["binary_sha256_final"] = None if pinned is None else sha256_of(pinned)
    document["summaries"] = summarize(document["rows"], document["warmups"], args.repetitions)
    environment = document["environment"]
    if build_record["status"] == "no artifact":
        document["status"] = "invalid: build reported no artifact"
    elif build_record["status"] == "artifact mismatch":
        document["status"] = "invalid: build artifact mismatch"
    elif build_record["status"] == "ambiguous":
        document["status"] = "invalid: build artifact ambiguous"
    elif build_record["status"] not in ("ok", "skipped"):
        document["status"] = "invalid: build failed"
    elif pinned is None:
        document["status"] = "invalid: binary could not be pinned"
    elif build_record["status"] == "skipped":
        document["status"] = "invalid: binary provenance unverified"
    elif not environment["rustc"] or not environment["cargo"] or any(
        override is not None and not override.get("disabled") and override["sha256"] is None for override in environment["compiler_overrides"].values()
    ) or (not args.skip_build and not build_record.get("compiler_chain")) or any(
        member["sha256"] is None for member in (build_record.get("compiler_chain") or [])
    ):
        document["status"] = "invalid: compiler identity unavailable"
    elif environment["source_commit"] is None or environment["worktree_dirty"] is not False or not all(environment["source_sha256"].values()) or (
        (environment["rustflags"]["CARGO_BUILD_TARGET"] or "").endswith(".json") and environment["rustflags"]["target_spec_sha256"] is None
    ) or (
        (build_record.get("target") or "").endswith(".json") and build_record.get("target_spec_sha256") is None
    ) or (
        build_record.get("target") and not build_record["target"].endswith(".json")
        and build_record.get("target_spec_sha256") is None and build_record.get("target_is_builtin") is not True
    ) or any(digest is None for digest in environment["rustflags"]["external_inputs"].values()):
        document["status"] = "invalid: source identity not reproducible"
    elif environment["binary_sha256_final"] != environment["binary_sha256"]:
        document["status"] = "invalid: binary changed during the campaign"
    else:
        document["status"] = "complete"
    if document["status"] != "complete":
        for summary in document["summaries"]:
            summary["selected"] = "unresolved"
    write_document(args.out, document)
    shutil.rmtree(staging, ignore_errors=True)
    for summary in document["summaries"]:
        print(f'{summary["workload"]["name"]}: lowest median {summary["lowest_median"]}, selected {summary["selected"]}, failures {summary["failures"]}, warmup failures {summary["warmup_failures"]}')


if __name__ == "__main__":
    main()
