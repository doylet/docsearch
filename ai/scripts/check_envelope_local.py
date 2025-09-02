#!/usr/bin/env python3
import json, subprocess, sys, pathlib
root = pathlib.Path(__file__).resolve().parents[2]
envp = root/".ai"/"envelope.json"
if not envp.exists():
    sys.exit("Missing .ai/envelope.json (create it or push with --no-verify).")
env = json.loads(envp.read_text())

def allowed_paths(e):
    s=set()
    for d in e.get("discovery", []):
        p=d.get("path");  s.update([p] if p else [])
    for c in e.get("changes", []):
        p=c.get("path");  s.update([p] if p else [])
    for t in e.get("tests", []):
        for k in ("path","golden"):
            p=t.get(k); s.update([p] if p else [])
    return s

allowed = allowed_paths(env)
# Compare against diff from merge-base with origin/main
try:
    merge_base = subprocess.check_output(["git","merge-base","HEAD","origin/main"]).decode().strip()
except subprocess.CalledProcessError:
    merge_base = "HEAD~1"
changed = subprocess.check_output(["git","diff","--name-only", f"{merge_base}...HEAD"]).decode().splitlines()
off = [f for f in changed if not any(f==a or f.startswith(a.rstrip("*")) for a in allowed)]
if off:
    print("Files outside envelope scope:")
    for f in off: print(" -", f)
    sys.exit(1)
limit = env.get("limits",{}).get("files_touched")
if isinstance(limit,int) and len(changed)>limit:
    sys.exit(f"Changed files {len(changed)} exceed declared limit {limit}.")
print("Local envelope scope OK.")
