import json

import pytest

import report
from m4_supplements import FullRun, Observation, soak_result


@pytest.mark.parametrize("failure", ["none", "decline", "crash", "unrecovered"])
def test_soak_classification(failure: str) -> None:
    data = report.ReportTests().record().model_dump(mode="json")
    data["window"] = {"start_ms": 0, "end_ms": 600000}
    data["sender"]["stats_file_samples"] = [
        {
            "t_ms": 600000,
            "document": {
                "connections": [
                    {
                        "conn_id": str(i),
                        "health": "down"
                        if failure == "unrecovered" and i == 1
                        else "healthy",
                    }
                    for i in range(3)
                ]
            },
        }
    ]
    data["raw"] = {
        "stats_csv_path": "/synthetic/receiver.csv",
        "sink_series": [
            {
                "t_ms": (i + 1) * 60000,
                "duration_ms": 60000,
                "bytes": 10000 - i * 100 if failure == "decline" else 10000,
            }
            for i in range(10)
        ],
    }
    run = FullRun.model_validate_json(json.dumps(data))
    observation = Observation(
        duration_seconds=610,
        sender_stderr_lines=10,
        sender_alive=failure != "crash",
        receiver_alive=True,
        caller_alive=True,
        listener_alive=True,
    )
    result = soak_result(run, observation)
    assert result.passed == (failure == "none")
    assert result.crashes == int(failure == "crash")
    assert result.unrecovered_links == int(failure == "unrecovered")
    assert result.monotonic_goodput_decline == (failure == "decline")
