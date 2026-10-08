# Читает stream-json `claude -p`, печатает «yes|no<TAB>цена»: загружен ли direktor.
import json, sys
loaded, cost = "no", "?"
for line in sys.stdin:
    try:
        j = json.loads(line)
    except ValueError:
        continue
    if j.get("type") == "assistant":
        for c in j["message"]["content"]:
            if c.get("type") == "tool_use" and c["name"] == "Skill" and c["input"].get("skill") == "direktor":
                loaded = "yes"
    elif j.get("type") == "result":
        cost = f"{j.get('total_cost_usd', 0):.4f}"
print(f"{loaded}\t{cost}")
