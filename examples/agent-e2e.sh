#!/usr/bin/env bash
# Example headless agent run. Fake names only. Never files or serves.
# Uses the same Rust engine as the desktop window.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CASE="$ROOT/tmp-agent-e2e"
rm -rf "$CASE"
CLI=(npm run --silent cli -- --json)

assert_envelope() {
  python3 -c '
import json, sys
raw = sys.stdin.read()
obj = json.loads(raw)
for key in ("ok", "never_files", "never_serves", "issues", "data", "notice"):
    assert key in obj, "missing envelope key %s in %s" % (key, raw[:400])
assert obj["never_files"] is True
assert obj["never_serves"] is True
print(raw)
'
}

echo "== create =="
"${CLI[@]}" case create --path "$CASE" --title "Example v. Sample County Clerk" --kind reconsideration --pack nj-app-div | assert_envelope

echo "== import markdown =="
"${CLI[@]}" draft import "$CASE" --md "$ROOT/fixtures/sample-reconsideration.md" | assert_envelope

echo "== evidence =="
printf 'Order granting dismissal (fake exhibit)\n' > "$CASE/order-excerpt.txt"
"${CLI[@]}" evidence add "$CASE" --file "$CASE/order-excerpt.txt" --description "Dkt. 12" | assert_envelope

echo "== check fact =="
"${CLI[@]}" check fact "$CASE" | python3 -c '
import json, sys
obj = json.loads(sys.stdin.read())
assert obj["never_files"] is True
assert obj["ok"] is False, "facts must start without invented cites"
assert obj["issues"], "expected fact issues"
print("fact issues:", len(obj["issues"]))
'

echo "== check authority =="
"${CLI[@]}" check authority "$CASE" | python3 -c '
import json, sys
obj = json.loads(sys.stdin.read())
assert obj["never_files"] is True
data = obj["data"]
assert data.get("invented") is False
cites = " ".join(r.get("cite","") for r in data.get("rows", []))
assert "2:11-6" in cites, cites
print("authority cites:", cites)
'

echo "== check deadline =="
"${CLI[@]}" check deadline "$CASE" --pack nj-app-div --rule reconsideration-appellate --trigger-date 2026-01-09 | python3 -c '
import json, sys
obj = json.loads(sys.stdin.read())
assert obj["never_files"] is True
assert obj["data"]["due"] == "2026-01-20", obj["data"]
print("due", obj["data"]["due"])
'

echo "== check all =="
"${CLI[@]}" check all "$CASE" --pack nj-app-div --rule reconsideration-appellate --trigger-date 2026-01-09 | python3 -c '
import json, sys
obj = json.loads(sys.stdin.read())
assert obj["never_files"] is True and obj["never_serves"] is True
assert obj["ok"] is False
print("check-all issues:", len(obj["issues"]))
'

echo "== exhibit index =="
"${CLI[@]}" evidence index "$CASE" | python3 -c '
import json, sys
obj = json.loads(sys.stdin.read())
letters = [i.get("letter") for i in obj["data"]["items"]]
assert "A" in letters, letters
print("exhibits", letters)
'

echo "== playbook =="
"${CLI[@]}" docs playbook | python3 -c '
import json, sys
obj = json.loads(sys.stdin.read())
assert "Never invent" in obj["data"]
print("playbook ok")
'

echo "== export =="
"${CLI[@]}" export court-md "$CASE" | python3 -c '
import json, sys, os
obj = json.loads(sys.stdin.read())
assert obj["never_files"] is True
docx = obj["data"]["docx"]
pdf = obj["data"]["pdf"]
assert os.path.isfile(docx), docx
assert os.path.isfile(pdf), pdf
print("docx", docx)
print("pdf", pdf)
'

echo "Done. The app did not file or serve. Folder: $CASE"
