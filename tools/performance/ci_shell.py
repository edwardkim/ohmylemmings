#!/usr/bin/env python3
"""GNU make SHELL adapter: preserve commands/results and append scoped wall/CPU/RSS data."""
import json
import resource
import subprocess
import sys
import time


def run(command):
    started=time.perf_counter_ns()
    result=subprocess.run(['bash','-c',command])
    elapsed=time.perf_counter_ns()-started
    usage=resource.getrusage(resource.RUSAGE_CHILDREN)
    print('PERF_COMMAND '+json.dumps(dict(command=command,returncode=result.returncode,
          wall_ns=elapsed,user_seconds=usage.ru_utime,system_seconds=usage.ru_stime,
          child_peak_rss_kib=usage.ru_maxrss,scope='whole check/build command; includes process, capture and I/O costs')),flush=True)
    return result.returncode


if __name__=='__main__':
    if len(sys.argv)!=3 or sys.argv[1]!='-c':raise SystemExit('ci_shell.py -c COMMAND')
    raise SystemExit(run(sys.argv[2]))
