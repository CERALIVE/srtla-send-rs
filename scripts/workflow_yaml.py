from __future__ import annotations

from dataclasses import dataclass

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


def mapping(node: Node) -> NodeMap:
    match node:
        case MappingNode(value=pairs):
            return NodeMap(tuple((scalar(key), value) for key, value in pairs))
        case ScalarNode() | SequenceNode():
            raise WorkflowFormatError("expected a YAML mapping")
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def scalar(node: Node) -> str:
    match node:
        case ScalarNode(value=value):
            return value
        case MappingNode() | SequenceNode():
            raise WorkflowFormatError("expected a YAML scalar")
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def sequence(node: Node) -> tuple[Node, ...]:
    match node:
        case SequenceNode(value=items):
            return tuple(items)
        case ScalarNode() | MappingNode():
            raise WorkflowFormatError("expected a YAML sequence")
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def optional_scalar(node_map: NodeMap, key: str) -> str:
    node = node_map.get(key)
    return "" if node is None else scalar(node)


def string_pairs(node: Node | None) -> tuple[tuple[str, str], ...]:
    if node is None:
        return ()
    return tuple((key, scalar(value)) for key, value in mapping(node).entries)


def permissions(
    node: Node | None, inherited: frozenset[tuple[str, str]] = frozenset()
) -> frozenset[tuple[str, str]]:
    if node is None:
        return inherited
    match node:
        case MappingNode():
            return frozenset(string_pairs(node))
        case ScalarNode():
            access = scalar(node)
            if access not in ("read-all", "write-all"):
                raise WorkflowFormatError(f"unsupported permissions value: {access}")
            return frozenset((("*", access.removesuffix("-all")),))
        case SequenceNode():
            raise WorkflowFormatError("permissions must be a mapping or read/write-all")
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def scalar_values(node: Node) -> tuple[str, ...]:
    match node:
        case ScalarNode(value=value):
            return (value,)
        case SequenceNode(value=items):
            return tuple(value for item in items for value in scalar_values(item))
        case MappingNode(value=pairs):
            return tuple(value for _key, item in pairs for value in scalar_values(item))
        case _:
            raise WorkflowFormatError("unsupported YAML node")


def uses_secret(node: Node | None) -> bool:
    return node is not None and any(
        "${{" in value and "secrets" in value.casefold()
        for value in scalar_values(node)
    )
