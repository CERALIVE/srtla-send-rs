from __future__ import annotations

import shlex
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

import yaml
from yaml import SafeLoader
from yaml.nodes import MappingNode, Node, ScalarNode, SequenceNode


class WorkflowFormatError(RuntimeError):
    pass


@dataclass(frozen=True, slots=True)
class NodeMap:
    entries: tuple[tuple[str, Node], ...]

    def get(self, key: str) -> Node | None:
        return next((value for name, value in self.entries if name == key), None)

    def require(self, key: str) -> Node:
        value = self.get(key)
        if value is None:
            raise WorkflowFormatError(f"missing required key: {key}")
        return value

    def keys(self) -> frozenset[str]:
        return frozenset(name for name, _value in self.entries)


@dataclass(frozen=True, slots=True)
class ShellCommand:
    executable: str
    prefix: tuple[str, ...]
    arguments: tuple[str, ...]

    def has_arguments(self, *required: str) -> bool:
        return all(argument in self.arguments for argument in required)

    @property
    def is_bounded(self) -> bool:
        return "timeout" in self.prefix

    @property
    def is_mutating_package_publish(self) -> bool:
        return (
            self.executable in ("npm", "pnpm")
            and self.has_arguments("publish")
            and "--dry-run" not in self.arguments
        )

    @property
    def is_dry_run_package_publish(self) -> bool:
        return self.executable in ("npm", "pnpm") and self.has_arguments(
            "publish", "--dry-run"
        )


@dataclass(frozen=True, slots=True)
class Step:
    action: str | None
    commands: tuple[ShellCommand, ...]
    environment: tuple[tuple[str, str], ...]

    def environment_value(self, key: str) -> str | None:
        return next((value for name, value in self.environment if name == key), None)


@dataclass(frozen=True, slots=True)
class Job:
    job_id: str
    needs: frozenset[str]
    condition: str
    permissions: frozenset[tuple[str, str]]
    steps: tuple[Step, ...]
    allows_failure: bool

    def commands(self, executable: str) -> tuple[ShellCommand, ...]:
        return tuple(
            command
            for step in self.steps
            for command in step.commands
            if command.executable == executable
        )

    def has_command(self, executable: str, *arguments: str) -> bool:
        return any(
            command.has_arguments(*arguments) for command in self.commands(executable)
        )

    @property
    def mutates_external_release_state(self) -> bool:
        publication_actions = (
            "softprops/action-gh-release@",
            "peter-evans/repository-dispatch@",
        )
        return any(
            (step.action is not None and step.action.startswith(publication_actions))
            or any(command.is_mutating_package_publish for command in step.commands)
            for step in self.steps
        )


@dataclass(frozen=True, slots=True)
class Workflow:
    triggers: frozenset[str]
    jobs: tuple[Job, ...]

    def job(self, job_id: str) -> Job:
        job = next(
            (candidate for candidate in self.jobs if candidate.job_id == job_id), None
        )
        if job is None:
            raise WorkflowFormatError(f"unknown job: {job_id}")
        return job

    @property
    def publication_jobs(self) -> tuple[Job, ...]:
        return tuple(job for job in self.jobs if job.mutates_external_release_state)


class JobStatus(StrEnum):
    SUCCESS = "success"
    FAILURE = "failure"
    SKIPPED = "skipped"


@dataclass(frozen=True, slots=True)
class Simulation:
    statuses: tuple[tuple[str, JobStatus], ...]

    def status(self, job_id: str) -> JobStatus:
        status = next((value for name, value in self.statuses if name == job_id), None)
        if status is None:
            raise WorkflowFormatError(f"simulation omitted job: {job_id}")
        return status


def _mapping(node: Node) -> NodeMap:
    match node:
        case MappingNode(value=pairs):
            entries: list[tuple[str, Node]] = []
            for key_node, value_node in pairs:
                entries.append((_scalar(key_node), value_node))
            return NodeMap(tuple(entries))
        case ScalarNode() | SequenceNode():
            raise WorkflowFormatError("expected a YAML mapping")
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def _scalar(node: Node) -> str:
    match node:
        case ScalarNode(value=value):
            return value
        case MappingNode() | SequenceNode():
            raise WorkflowFormatError("expected a YAML scalar")
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def _sequence(node: Node) -> tuple[Node, ...]:
    match node:
        case SequenceNode(value=items):
            return tuple(items)
        case ScalarNode() | MappingNode():
            raise WorkflowFormatError("expected a YAML sequence")
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def _optional_scalar(mapping: NodeMap, key: str) -> str:
    node = mapping.get(key)
    return "" if node is None else _scalar(node)


def _string_pairs(node: Node | None) -> tuple[tuple[str, str], ...]:
    if node is None:
        return ()
    return tuple((key, _scalar(value)) for key, value in _mapping(node).entries)


def _commands(script: str) -> tuple[ShellCommand, ...]:
    supported = ("cargo", "npm", "pnpm", "bash", "uv")
    commands: list[ShellCommand] = []
    for raw_line in script.splitlines():
        line = raw_line.strip().removesuffix("\\").strip()
        if not line or line.startswith("#"):
            continue
        try:
            tokens = tuple(shlex.split(line))
        except ValueError as error:
            raise WorkflowFormatError(f"invalid shell command: {line}") from error
        executable = tokens[0]
        if executable in supported:
            commands.append(ShellCommand(executable, (), tokens[1:]))
        elif executable == "timeout" and "cargo" in tokens:
            index = tokens.index("cargo")
            commands.append(ShellCommand("cargo", tokens[:index], tokens[index + 1 :]))
    return tuple(commands)


def _step(node: Node) -> Step:
    mapping = _mapping(node)
    run = _optional_scalar(mapping, "run")
    return Step(
        action=_optional_scalar(mapping, "uses") or None,
        commands=_commands(run),
        environment=_string_pairs(mapping.get("env")),
    )


def _needs(node: Node | None) -> frozenset[str]:
    if node is None:
        return frozenset()
    match node:
        case ScalarNode():
            return frozenset((_scalar(node),))
        case SequenceNode():
            return frozenset(_scalar(item) for item in _sequence(node))
        case MappingNode():
            raise WorkflowFormatError("job needs must be a string or sequence")
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def _job(job_id: str, node: Node) -> Job:
    mapping = _mapping(node)
    steps_node = mapping.require("steps")
    return Job(
        job_id=job_id,
        needs=_needs(mapping.get("needs")),
        condition=_optional_scalar(mapping, "if"),
        permissions=frozenset(_string_pairs(mapping.get("permissions"))),
        steps=tuple(_step(step) for step in _sequence(steps_node)),
        allows_failure=_optional_scalar(mapping, "continue-on-error") == "true",
    )


def load_workflow(path: Path) -> Workflow:
    with path.open(encoding="utf-8") as handle:
        document = yaml.compose(handle, Loader=SafeLoader)
    if document is None:
        raise WorkflowFormatError(f"empty workflow: {path}")
    root = _mapping(document)
    trigger_node = root.require("on")
    match trigger_node:
        case ScalarNode():
            triggers = frozenset((_scalar(trigger_node),))
        case MappingNode():
            triggers = _mapping(trigger_node).keys()
        case SequenceNode():
            triggers = frozenset(_scalar(item) for item in _sequence(trigger_node))
        case _:
            raise WorkflowFormatError("unsupported trigger node")
    jobs = tuple(
        _job(job_id, node) for job_id, node in _mapping(root.require("jobs")).entries
    )
    return Workflow(triggers=triggers, jobs=jobs)


def transitive_needs(workflow: Workflow, job_id: str) -> frozenset[str]:
    found: set[str] = set()
    pending = list(workflow.job(job_id).needs)
    while pending:
        dependency = pending.pop()
        if dependency in found:
            continue
        found.add(dependency)
        pending.extend(workflow.job(dependency).needs)
    return frozenset(found)


def simulate(
    workflow: Workflow, initial: tuple[tuple[str, JobStatus], ...]
) -> Simulation:
    statuses = dict(initial)
    remaining = {job.job_id for job in workflow.jobs} - statuses.keys()
    while remaining:
        ready = next(
            (
                job_id
                for job_id in sorted(remaining)
                if workflow.job(job_id).needs <= statuses.keys()
            ),
            None,
        )
        if ready is None:
            raise WorkflowFormatError(
                f"cyclic or unknown dependency: {sorted(remaining)}"
            )
        dependencies = workflow.job(ready).needs
        statuses[ready] = (
            JobStatus.SKIPPED
            if any(
                statuses[dependency] is not JobStatus.SUCCESS
                for dependency in dependencies
            )
            else JobStatus.SUCCESS
        )
        remaining.remove(ready)
    return Simulation(tuple(sorted(statuses.items())))
