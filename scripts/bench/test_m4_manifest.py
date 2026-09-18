from pathlib import Path

import pytest

from report import EvidenceError


def test_exact_matrix_and_reserved_scope() -> None:
    from m4_manifest import build, validate

    manifest = build(Path("target/m4-srtla_send"))
    validate(manifest)
    assert len(manifest.cells) == 110
    assert sum(cell.runs for cell in manifest.cells) == 546
    assert sum(cell.covering for cell in manifest.cells) == 100
    assert all(cell.port != 4003 and cell.scenario != "M8" for cell in manifest.cells)


@pytest.mark.parametrize(
    "mutation", ["missing", "duplicate", "baseline", "fec", "n", "priority"]
)
def test_manifest_rejects_scope_corruption(mutation: str) -> None:
    from m4_manifest import build, validate

    manifest = build(Path("target/m4-srtla_send"))
    cells = list(manifest.cells)
    match mutation:
        case "missing":
            cells.pop(0)
        case "duplicate":
            cells[-1] = cells[0]
        case "baseline":
            cells[100] = cells[100].model_copy(update={"covering": True})
        case "fec":
            cells[-1] = cells[-1].model_copy(update={"covering": True})
        case "n":
            cells[0] = cells[0].model_copy(update={"runs": 3})
        case "priority":
            cells = [
                cell.model_copy(update={"priority_sidecar": None})
                if cell.scenario == "M6"
                else cell
                for cell in cells
            ]
        case _:
            raise AssertionError(mutation)
    with pytest.raises(EvidenceError):
        validate(manifest.model_copy(update={"cells": tuple(cells)}))
