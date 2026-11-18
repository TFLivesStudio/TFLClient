"""Compare two signed AppImages under the same hosted Linux desktop conditions."""

import json
import os
from pathlib import Path
import signal
import statistics
import subprocess
import sys
import tempfile
import time


def sample(session):
    processes = {}
    for stat in Path("/proc").glob("[0-9]*/stat"):
        try:
            parsed = stat.read_text()
            name, tail = parsed.split(" (", 1)[1].rsplit(") ", 1)
            fields = tail.split()
            if int(fields[3]) != session:
                continue
            memory = (stat.parent / "smaps_rollup").read_text().splitlines()
            pss = next(int(row.split()[1]) for row in memory if row.startswith("Pss:"))
            processes[(stat.parent.name, fields[19])] = (
                name, int(fields[11]) + int(fields[12]), pss
            )
        except (FileNotFoundError, ProcessLookupError, PermissionError, StopIteration):
            continue
    return time.monotonic(), processes


def measure(binary):
    with tempfile.TemporaryDirectory(prefix="tfl-idle-") as directory:
        profile = Path(directory)
        (profile / "TEST_PROFILE").touch()
        (profile / ".tflclient/settings").mkdir(parents=True)
        (profile / ".tflclient/settings/settings.tfl").write_text(
            json.dumps({"onboarded": True, "auto_updates": False, "discord_presence": False})
        )
        binary = Path(binary).resolve()
        binary.chmod(binary.stat().st_mode | 0o100)
        env = dict(os.environ, TFL_DATA_DIR=str(profile), APPIMAGE_EXTRACT_AND_RUN="1")
        with (profile / "launcher.log").open("wb") as log:
            app = subprocess.Popen(
                [binary], stdout=log, stderr=subprocess.STDOUT, env=env, start_new_session=True
            )
            try:
                for _ in range(45):
                    if app.poll() is not None:
                        raise RuntimeError("The launcher exited before its window opened")
                    if subprocess.run(
                        ["xdotool", "search", "--onlyvisible", "--name", "^TFL Client$"],
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL,
                        check=False,
                    ).returncode == 0:
                        break
                    time.sleep(1)
                else:
                    raise RuntimeError("The launcher window did not open")
                time.sleep(10)
                previous_time, previous = sample(app.pid)
                readings = []
                ticks_per_second = os.sysconf("SC_CLK_TCK")
                for _ in range(5):
                    time.sleep(2)
                    measured_at, current = sample(app.pid)
                    if not current or app.poll() is not None:
                        raise RuntimeError("No launcher processes remained to sample")
                    ticks = sum(
                        row[1] - previous[key][1] for key, row in current.items() if key in previous
                    )
                    readings.append({
                        "one_core_percent": ticks * 100 / ticks_per_second / (measured_at - previous_time),
                        "pss_mib": sum(row[2] for row in current.values()) / 1024,
                        "processes": sorted(row[0] for row in current.values()),
                    })
                    previous_time, previous = measured_at, current
                return {
                    "samples": readings,
                    "median_one_core_percent": statistics.median(row["one_core_percent"] for row in readings),
                    "median_pss_mib": statistics.median(row["pss_mib"] for row in readings),
                }
            finally:
                try:
                    os.killpg(app.pid, signal.SIGTERM)
                    app.wait(timeout=5)
                except ProcessLookupError:
                    pass
                except subprocess.TimeoutExpired:
                    os.killpg(app.pid, signal.SIGKILL)
                    app.wait()


if __name__ == "__main__":
    before, after = sys.argv[1:]
    print(json.dumps({
        "kernel": os.uname().release,
        "logical_cpus": os.cpu_count(),
        "warmup_seconds": 10,
        "sampling_seconds": 10,
        "scope": "Processes in the launcher session, including AppImage and WebKit helpers",
        "limitations": "Hosted Linux and an isolated idle profile. CPU excludes processes that exit between samples.",
        "baseline": measure(before),
        "candidate": measure(after),
    }, indent=2))
