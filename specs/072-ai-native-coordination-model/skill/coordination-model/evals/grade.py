#!/usr/bin/env python3
"""Grade coordination model behavioral eval results.

Checks agent output (trace.md) for structural evidence of coordination
primitive behavior. Does NOT judge output quality — only process adherence.

Usage:
    python grade.py results/speculative-swarm-ratelimiter/
    python grade.py results/  # grade all evals
    python grade.py --list     # list available eval definitions
"""
import argparse
import json
import re
import sys
from pathlib import Path

EVALS_DIR = Path(__file__).resolve().parent
PROMPTS_DIR = EVALS_DIR / "prompts"


# ---------------------------------------------------------------------------
# Marker detection heuristics
# ---------------------------------------------------------------------------
# These look for structural evidence in the agent's trace output.
# They're intentionally generous — we check for process, not polish.

MARKER_PATTERNS: dict[str, list[re.Pattern]] = {
    "fork_evidence": [
        re.compile(r"(?:strategy|approach|variant|option|design)\s*[#\d:]\s", re.I),
        re.compile(r"(?:fork|diverge|branch|alternative)\s", re.I),
        re.compile(r"##\s*(?:Strategy|Approach|Option|Variant|Design)\s", re.I),
    ],
    "observe_evidence": [
        re.compile(r"(?:compar|evaluat|assess|characterist|trade.?off)", re.I),
        re.compile(r"\|.*\|.*\|", re.I),  # table row (comparison table)
    ],
    "convergence_evidence": [
        re.compile(r"(?:converg|similar|overlap|redundan)", re.I),
    ],
    "prune_evidence": [
        re.compile(r"(?:eliminat|prun|remov|discard|reject|drop)\w*\s", re.I),
        re.compile(r"(?:weaker|worst|least|inferior)", re.I),
    ],
    "merge_evidence": [
        re.compile(r"(?:merg|fus|combin|synthesi|integrat)\w*\s", re.I),
        re.compile(r"(?:from\s+(?:strategy|approach|option))", re.I),
        re.compile(r"(?:fragment|borrow|take from)", re.I),
    ],
    "escalation_evidence": [
        re.compile(r"(?:round|phase|level)\s*[#\d:]", re.I),
        re.compile(r"##\s*Round\s", re.I),
    ],
    "critic_specificity": [
        re.compile(r"(?:critic|attack|finding|issue|vulnerabilit)", re.I),
    ],
    "generator_response": [
        re.compile(r"(?:fix|patch|harden|address|resolv)", re.I),
        re.compile(r"```\w+", re.I),  # code block (concrete fix)
    ],
    "termination_condition": [
        re.compile(r"(?:terminat|stop|conclud|no\s+(?:new|more)\s+issue)", re.I),
        re.compile(r"(?:consecutive\s+clean|quality\s+threshold)", re.I),
    ],
    "progressive_difficulty": [
        re.compile(r"(?:surface|edge.case|adversarial|semantic|architect)", re.I),
    ],
    "decomposition_evidence": [
        re.compile(r"(?:decompos|split|sub.?problem|sub.?task|sub.?agent)", re.I),
        re.compile(r"(?:├|└|│|→)", re.I),  # tree chars
    ],
    "scope_isolation": [
        re.compile(r"(?:scope|boundar|input|output|not.?handl|owns?|responsible)", re.I),
    ],
    "recursive_depth": [
        re.compile(r"(?:further\s+(?:split|decompos)|sub.?sub|level\s*[23])", re.I),
    ],
    "independent_solutions": [
        re.compile(r"(?:leaf|independent|self.?contained|isolated)", re.I),
    ],
    "reunification_evidence": [
        re.compile(r"(?:reunif|recompos|assembl|integrat|compose)", re.I),
    ],
    "dag_evidence": [
        re.compile(r"(?:DAG|graph|depend|link|reference|connect)", re.I),
        re.compile(r"(?:→|--|>>|flowchart|mermaid)", re.I),
    ],
    "gap_detection": [
        re.compile(r"(?:gap|missing|unanswer|unknown|open\s+question)", re.I),
    ],
    "reactive_spawn": [
        re.compile(r"(?:spawn|creat|fill|address)\w*\s+(?:to|for|because)", re.I),
        re.compile(r"(?:triggered\s+by|in\s+response\s+to)", re.I),
    ],
    "reactive_evidence": [
        re.compile(r"(?:react|respond|trigger|watch|detect|marker)", re.I),
    ],
    "proportionality": [
        re.compile(r"(?:targeted|minimal|proportional|focused|specific)", re.I),
        re.compile(r"(?:patch|small|scoped)", re.I),
    ],
    "debounce_reasoning": [
        re.compile(r"(?:debounce|suppress|separate|independent|storm)", re.I),
    ],
    "decay_evidence": [
        re.compile(r"(?:decay|expir|urgency|time.?sensitiv|priority|stale)", re.I),
    ],
    "stage_sequence": [
        re.compile(r"(?:stage\s*[123]|step\s*[123]|phase\s*[123])", re.I),
        re.compile(r"(?:explore|harden|maintain)", re.I),
    ],
    "stage_handoff": [
        re.compile(r"(?:handoff|hand.?off|input\s+from|output\s+to|feeds?\s+into)", re.I),
        re.compile(r"(?:from\s+stage|to\s+stage|previous\s+stage)", re.I),
    ],
    "swarm_evidence": [
        re.compile(r"(?:swarm|fork|strateg)", re.I),
    ],
    "adversarial_evidence": [
        re.compile(r"(?:adversarial|critic|generator|round)", re.I),
    ],
    "stigmergic_evidence": [
        re.compile(r"(?:stigmergic|marker|watch|artifact|react)", re.I),
    ],
    "composition_coherence": [
        re.compile(r"(?:builds?\s+on|based\s+on|from\s+(?:stage|previous))", re.I),
    ],
    "spawn_evidence": [
        re.compile(r"(?:node|knowledge\s+(?:block|node|area)|domain)", re.I),
    ],
}


def detect_marker(marker_name: str, text: str) -> dict:
    """Check whether a marker is evidenced in the text."""
    patterns = MARKER_PATTERNS.get(marker_name, [])
    hits = []
    for pat in patterns:
        matches = pat.findall(text)
        if matches:
            hits.extend(matches[:3])  # cap per pattern
    return {
        "detected": len(hits) > 0,
        "hit_count": len(hits),
        "samples": hits[:5],
    }


def count_distinct_sections(text: str, pattern: re.Pattern) -> int:
    """Count how many distinct heading-level sections match a pattern."""
    return len(pattern.findall(text))


def load_eval_spec(prompt_path: Path) -> dict | None:
    """Extract grading markers JSON from a prompt file."""
    content = prompt_path.read_text()
    # Find the JSON block after "Grading markers"
    match = re.search(r"```json\s*\n(\{.*?\})\s*\n```", content, re.S)
    if match:
        try:
            return json.loads(match.group(1))
        except json.JSONDecodeError:
            return None
    return None


def grade_trace(trace_path: Path, eval_spec: dict) -> dict:
    """Grade a trace file against an eval spec's markers."""
    text = trace_path.read_text()
    results = {}
    markers = eval_spec.get("markers", {})
    required_count = 0
    passed_count = 0

    for marker_name, marker_def in markers.items():
        is_required = marker_def.get("required", False)
        detection = detect_marker(marker_name, text)

        result = {
            "check": marker_def.get("check", ""),
            "required": is_required,
            "detected": detection["detected"],
            "hit_count": detection["hit_count"],
            "samples": detection["samples"],
        }
        results[marker_name] = result

        if is_required:
            required_count += 1
            if detection["detected"]:
                passed_count += 1

    return {
        "primitive": eval_spec.get("primitive", "unknown"),
        "markers": results,
        "required_total": required_count,
        "required_passed": passed_count,
        "pass": passed_count == required_count,
    }


def grade_eval_dir(eval_dir: Path) -> dict:
    """Grade all results in an eval directory."""
    trace_path = eval_dir / "trace.md"
    if not trace_path.exists():
        return {"error": f"No trace.md found in {eval_dir}"}

    # Find matching prompt
    eval_name = eval_dir.name
    prompt_path = PROMPTS_DIR / f"{eval_name}.md"
    if not prompt_path.exists():
        return {"error": f"No prompt definition found: {prompt_path}"}

    eval_spec = load_eval_spec(prompt_path)
    if not eval_spec:
        return {"error": f"Could not extract grading markers from {prompt_path}"}

    result = grade_trace(trace_path, eval_spec)

    # Write metadata
    metadata_path = eval_dir / "metadata.json"
    metadata_path.write_text(json.dumps(result, indent=2) + "\n")

    return result


def list_evals():
    """List all available eval definitions."""
    print("Available behavioral evals:\n")
    for prompt_file in sorted(PROMPTS_DIR.glob("*.md")):
        spec = load_eval_spec(prompt_file)
        primitive = spec.get("primitive", "?") if spec else "?"
        markers = spec.get("markers", {}) if spec else {}
        required = sum(1 for m in markers.values() if m.get("required"))
        print(f"  {prompt_file.stem}")
        print(f"    primitive: {primitive}")
        print(f"    markers: {len(markers)} ({required} required)")
        print()


def main():
    parser = argparse.ArgumentParser(description="Grade coordination model behavioral evals.")
    parser.add_argument("path", nargs="?", help="Path to results/<eval-name>/ or results/")
    parser.add_argument("--list", action="store_true", help="List available eval definitions")
    parser.add_argument("--json", action="store_true", help="Output results as JSON")
    args = parser.parse_args()

    if args.list:
        list_evals()
        return

    if not args.path:
        parser.print_help()
        sys.exit(1)

    target = Path(args.path)
    if not target.exists():
        print(f"Path not found: {target}", file=sys.stderr)
        sys.exit(1)

    # Single eval dir or parent of multiple
    if (target / "trace.md").exists():
        eval_dirs = [target]
    else:
        eval_dirs = sorted(d for d in target.iterdir() if d.is_dir() and (d / "trace.md").exists())

    if not eval_dirs:
        print(f"No eval results found in {target} (expected trace.md files)", file=sys.stderr)
        sys.exit(1)

    all_results = {}
    any_fail = False

    for eval_dir in eval_dirs:
        result = grade_eval_dir(eval_dir)
        all_results[eval_dir.name] = result

        if args.json:
            continue

        # Human-readable output
        print(f"{'=' * 60}")
        print(f"Eval: {eval_dir.name}")

        if "error" in result:
            print(f"  ERROR: {result['error']}")
            any_fail = True
            continue

        print(f"  Primitive: {result['primitive']}")
        print(f"  Result: {'PASS' if result['pass'] else 'FAIL'}")
        print(f"  Required: {result['required_passed']}/{result['required_total']}")
        print()

        for marker_name, marker_result in result["markers"].items():
            status = "✓" if marker_result["detected"] else "✗"
            req = " (required)" if marker_result["required"] else ""
            print(f"  {status} {marker_name}{req}")
            print(f"    {marker_result['check']}")
            if marker_result["detected"]:
                print(f"    hits: {marker_result['hit_count']}")
            print()

        if not result["pass"]:
            any_fail = True

    if args.json:
        print(json.dumps(all_results, indent=2))

    print(f"\n{'=' * 60}")
    total = len(all_results)
    passed = sum(1 for r in all_results.values() if r.get("pass"))
    print(f"Total: {passed}/{total} passed")

    sys.exit(1 if any_fail else 0)


if __name__ == "__main__":
    main()
