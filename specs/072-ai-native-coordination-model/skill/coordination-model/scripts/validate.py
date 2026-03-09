#!/usr/bin/env python3
"""Validate coordination model artifacts against JSON Schema.

Usage:
    python validate.py playbook.yaml                    # validate a playbook
    python validate.py --schema conformance conf.json   # validate conformance
    python validate.py --schema primitives prim.json    # validate a primitive config
"""
import argparse
import json
import sys
from pathlib import Path

try:
    import jsonschema
except ImportError:
    print("Missing dependency: pip install jsonschema", file=sys.stderr)
    sys.exit(1)

try:
    import yaml
    HAS_YAML = True
except ImportError:
    HAS_YAML = False

SCHEMA_DIR = Path(__file__).resolve().parent.parent / "references"

SCHEMAS = {
    "playbook": "playbook.schema.json",
    "conformance": "conformance.schema.json",
    "primitives": "primitives.schema.json",
    "operations": "operations.schema.json",
}

ANTI_PATTERNS = [
    ("speculative-swarm", "speculative-swarm"),
    ("generative-adversarial", "generative-adversarial"),
]


def load_file(path: str) -> dict:
    p = Path(path)
    text = p.read_text()
    if p.suffix in (".yaml", ".yml"):
        if not HAS_YAML:
            print("YAML support requires: pip install pyyaml", file=sys.stderr)
            sys.exit(1)
        return yaml.safe_load(text)
    return json.loads(text)


def check_anti_patterns(data: dict) -> list[str]:
    warnings = []
    playbook = data.get("playbook", data)
    stages = playbook.get("stages", [])
    rules = playbook.get("composition_rules", [])

    stage_map = {s["name"]: s["primitive"] for s in stages if "name" in s and "primitive" in s}

    for rule in rules:
        outer_prim = stage_map.get(rule.get("outer"))
        inner_prim = stage_map.get(rule.get("inner"))
        if (outer_prim, inner_prim) in ANTI_PATTERNS:
            warnings.append(
                f"Anti-pattern: {outer_prim} inside {outer_prim} "
                f"(stages '{rule['outer']}' → '{rule['inner']}')"
            )

    # Check stigmergic without debounce
    for stage in stages:
        if stage.get("primitive") == "stigmergic":
            config = stage.get("config", {})
            if "reaction_debounce" not in config:
                warnings.append(
                    f"Anti-pattern: stigmergic stage '{stage['name']}' "
                    f"has no reaction_debounce — risk of reaction storms"
                )

    return warnings


def main():
    parser = argparse.ArgumentParser(description="Validate coordination model artifacts.")
    parser.add_argument("file", help="Path to the file to validate (JSON or YAML)")
    parser.add_argument(
        "--schema",
        choices=list(SCHEMAS.keys()),
        default="playbook",
        help="Which schema to validate against (default: playbook)",
    )
    args = parser.parse_args()

    schema_path = SCHEMA_DIR / SCHEMAS[args.schema]
    if not schema_path.exists():
        print(f"Schema not found: {schema_path}", file=sys.stderr)
        sys.exit(1)

    schema = json.loads(schema_path.read_text())
    data = load_file(args.file)

    try:
        jsonschema.validate(instance=data, schema=schema)
    except jsonschema.ValidationError as e:
        print(f"FAIL: {e.message}", file=sys.stderr)
        print(f"  Path: {'.'.join(str(p) for p in e.absolute_path)}", file=sys.stderr)
        sys.exit(1)

    print(f"OK: validates against {args.schema} schema")

    if args.schema == "playbook":
        warnings = check_anti_patterns(data)
        for w in warnings:
            print(f"WARNING: {w}", file=sys.stderr)

    if args.schema == "conformance":
        must = data.get("runtime", {}).get("must", {})
        ops = must.get("abstract_operations", {})
        missing_ops = [op for op in ["spawn", "fork", "merge", "observe", "convergence", "prune"] if not ops.get(op)]
        missing_caps = [k for k in ["dynamic_lifecycle", "state_observability", "budget_enforcement",
                                     "composable_patterns", "trace_capture", "declarative_playbooks"]
                        if not must.get(k)]
        if missing_ops or missing_caps:
            print("NON-CONFORMANT:", file=sys.stderr)
            for op in missing_ops:
                print(f"  Missing operation: {op}", file=sys.stderr)
            for cap in missing_caps:
                print(f"  Missing capability: {cap}", file=sys.stderr)
            sys.exit(1)
        print("CONFORMANT: all MUST requirements satisfied")


if __name__ == "__main__":
    main()
