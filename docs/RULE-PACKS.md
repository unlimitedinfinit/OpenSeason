# Rule pack authoring guide

A rule pack is a JSON file. New states are data, not Rust. The pack is a deadline table, not legal advice. Every pack must set `never_files` and `never_serves` to true.

## Validate

```bash
npm run cli -- --json packs validate templates/rule-packs/nj-app-div.json
```

The validator requires `id`, `name`, `never_files`, `never_serves`, and at least one deadline with `id` and `rule`.

## Shape

```json
{
  "id": "xx-app",
  "name": "Sample State Appellate",
  "jurisdiction": "XX",
  "court": "Sample Court of Appeals",
  "holiday_set": "us-federal",
  "source_url": "https://example.test/rules",
  "source_retrieved_on": "2026-10-14",
  "source_note": "Confirm the current rules before relying on a date.",
  "never_files": true,
  "never_serves": true,
  "deadlines": [
    {
      "id": "reconsideration-appellate",
      "label": "Motion for reconsideration",
      "rule": "R. 00-0",
      "trigger": "entry",
      "days": 10,
      "count": "calendar",
      "weekend_holiday_roll": "next_business",
      "enlargeable_under": null,
      "tolls_appeal_under": null,
      "notes": "Confirm the current rule text."
    }
  ],
  "knowledge_base": []
}
```

`holiday_set` is `us-federal` or `us-nj`. Put the file in `templates/rule-packs/` and add a `RulePack::builtin` arm if you want it bundled.

Colorado C.A.R. 21 is a public-rule scaffold. Mandamus-Colorado-09-2026 is private and was not readable here.
