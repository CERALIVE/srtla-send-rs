from __future__ import annotations

import shlex
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

import yaml
from yaml import SafeLoader
from yaml.nodes import MappingNode, Node, ScalarNode, SequenceNode

from workflow_yaml import (
    WorkflowFormatError,
    mapping,
    optional_scalar,
    permissions,
    scalar,
    sequence,
    string_pairs,
    uses_secret,
)


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
    def is_dry_run_package_publish(self) -> bool:
        return self.executable in ("npm", "pnpm") and self.has_arguments(
            "publish", "--dry-run"
        )


@dataclass(frozen=True, slots=True)
class Step:
    name: str
    action: str | None
    commands: tuple[ShellCommand, ...]
    environment: tuple[tuple[str, str], ...]

    def environment_value(self, key: str) -> str | None:
        return next((value for name, value in self.environment if name == key), None)


@dataclass(frozen=True, slots=True)
class Job:
    job_id: str
    name: str
    needs: frozenset[str]
    condition: str
    permissions: frozenset[tuple[str, str]]
    steps: tuple[Step, ...]
    allows_failure: bool
    uses_secret: bool

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
    def has_external_mutation_authority(self) -> bool:
        return self.uses_secret or any(
            access == "write" for _scope, access in self.permissions
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
        return tuple(job for job in self.jobs if job.has_external_mutation_authority)


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
    node_map = mapping(node)
    run = optional_scalar(node_map, "run")
    return Step(
        name=optional_scalar(node_map, "name"),
        action=optional_scalar(node_map, "uses") or None,
        commands=_commands(run),
        environment=string_pairs(node_map.get("env")),
    )


def _needs(node: Node | None) -> frozenset[str]:
    if node is None:
        return frozenset()
    match node:
        case ScalarNode():
            return frozenset((scalar(node),))
        case SequenceNode():
            return frozenset(scalar(item) for item in sequence(node))
        case MappingNode():
            raise WorkflowFormatError("job needs must be a string or sequence")
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def _job(
    job_id: str,
    node: Node,
    inherited_permissions: frozenset[tuple[str, str]],
    inherited_secret: bool,
) -> Job:
    node_map = mapping(node)
    steps_node = node_map.require("steps")
    return Job(
        job_id=job_id,
        name=optional_scalar(node_map, "name"),
        needs=_needs(node_map.get("needs")),
        condition=optional_scalar(node_map, "if"),
        permissions=permissions(node_map.get("permissions"), inherited_permissions),
        steps=tuple(_step(step) for step in sequence(steps_node)),
        allows_failure=optional_scalar(node_map, "continue-on-error") == "true",
        uses_secret=inherited_secret or uses_secret(node),
    )


def load_workflow(path: Path) -> Workflow:
    with path.open(encoding="utf-8") as handle:
        document = yaml.compose(handle, Loader=SafeLoader)
    if document is None:
        raise WorkflowFormatError(f"empty workflow: {path}")
    root = mapping(document)
    permissions_node = root.get("permissions")
    inherited_permissions = (
        permissions(permissions_node)
        if permissions_node is not None
        else frozenset((("*", "write"),))
    )
    inherited_secret = uses_secret(root.get("env"))
    trigger_node = root.require("on")
    match trigger_node:
        case ScalarNode():
            triggers = frozenset((scalar(trigger_node),))
        case MappingNode():
            triggers = mapping(trigger_node).keys()
        case SequenceNode():
            triggers = frozenset(scalar(item) for item in sequence(trigger_node))
        case _:
            raise WorkflowFormatError("unsupported trigger node")
    jobs = tuple(
        _job(job_id, node, inherited_permissions, inherited_secret)
        for job_id, node in mapping(root.require("jobs")).entries
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
