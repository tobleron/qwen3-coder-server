#!/usr/bin/env python3
"""
Reliability verifier for Qwen3/OpenCode-compatible tool calling.

Runs repeated non-streaming chat completions and reports:
- tool-call success rate (required tool mode)
- malformed argument rate
- no-tool false positive rate
- loop-like output incidence
- latency stats
"""

import argparse
import json
import statistics
import time
from typing import Any, Dict, List, Tuple

import requests


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument("--base-url", default="http://localhost:8081/v1")
    p.add_argument("--model", default="cerebras-qwen3")
    p.add_argument("--required-trials", type=int, default=50)
    p.add_argument("--plain-trials", type=int, default=20)
    p.add_argument("--timeout", type=int, default=120)
    p.add_argument("--temperature", type=float, default=0.2)
    p.add_argument("--verbose", action="store_true")
    return p.parse_args()


def post_chat(base_url: str, payload: Dict[str, Any], timeout: int) -> Tuple[float, requests.Response]:
    t0 = time.time()
    r = requests.post(f"{base_url}/chat/completions", json=payload, timeout=timeout)
    return (time.time() - t0), r


def looks_loop_like(text: str) -> bool:
    if not text:
        return False
    low = text.lower()
    markers = [
        "i will now",
        "as mentioned above",
        "as previously",
        "repeating",
        "<|im_start|>",
        "<|im_end|>",
        "<tool_call>",
    ]
    if any(m in low for m in markers):
        return True

    lines = [ln.strip() for ln in text.splitlines() if ln.strip()]
    if len(lines) >= 6:
        uniq = len(set(lines))
        if uniq / len(lines) < 0.5:
            return True
    return False


def run_required_trials(args: argparse.Namespace) -> Dict[str, Any]:
    tools = [
        {
            "type": "function",
            "function": {
                "name": "bash",
                "description": "Execute bash commands",
                "parameters": {
                    "type": "object",
                    "properties": {"command": {"type": "string"}},
                    "required": ["command"],
                },
            },
        }
    ]

    ok = 0
    malformed_args = 0
    loop_like = 0
    http_fail = 0
    latencies: List[float] = []
    samples = []

    for i in range(args.required_trials):
        payload = {
            "model": args.model,
            "messages": [
                {"role": "user", "content": "Use the bash tool to run exactly: echo ok"}
            ],
            "tools": tools,
            "tool_choice": "required",
            "temperature": args.temperature,
            "max_tokens": 220,
            "stream": False,
        }

        try:
            dt, r = post_chat(args.base_url, payload, args.timeout)
            latencies.append(dt)
            if r.status_code != 200:
                http_fail += 1
                if args.verbose:
                    samples.append({"trial": i + 1, "status": r.status_code, "body": r.text[:180]})
                continue

            d = r.json()
            ch = d.get("choices", [{}])[0]
            msg = ch.get("message", {})
            finish_reason = ch.get("finish_reason")
            tool_calls = msg.get("tool_calls") or []
            content = msg.get("content") or ""

            trial_ok = finish_reason == "tool_calls" and len(tool_calls) > 0
            if trial_ok:
                ok += 1

            if looks_loop_like(content):
                loop_like += 1

            if tool_calls:
                try:
                    arg_str = tool_calls[0].get("function", {}).get("arguments", "{}")
                    parsed = json.loads(arg_str)
                    if not isinstance(parsed, dict) or "command" not in parsed or not isinstance(parsed["command"], str):
                        malformed_args += 1
                except Exception:
                    malformed_args += 1

            if args.verbose:
                samples.append(
                    {
                        "trial": i + 1,
                        "ok": trial_ok,
                        "finish_reason": finish_reason,
                        "tool_calls": len(tool_calls),
                        "content_preview": str(content).replace("\n", " ")[:120],
                    }
                )
        except Exception as e:
            http_fail += 1
            if args.verbose:
                samples.append({"trial": i + 1, "exception": str(e)[:180]})

    return {
        "trials": args.required_trials,
        "ok": ok,
        "http_fail": http_fail,
        "malformed_args": malformed_args,
        "loop_like": loop_like,
        "latencies": latencies,
        "samples": samples,
    }


def run_plain_trials(args: argparse.Namespace) -> Dict[str, Any]:
    ok = 0
    false_tool_calls = 0
    loop_like = 0
    http_fail = 0
    latencies: List[float] = []
    samples = []

    for i in range(args.plain_trials):
        payload = {
            "model": args.model,
            "messages": [{"role": "user", "content": "Answer in one short line: what is 2+2?"}],
            "temperature": args.temperature,
            "max_tokens": 80,
            "stream": False,
        }

        try:
            dt, r = post_chat(args.base_url, payload, args.timeout)
            latencies.append(dt)
            if r.status_code != 200:
                http_fail += 1
                if args.verbose:
                    samples.append({"trial": i + 1, "status": r.status_code, "body": r.text[:180]})
                continue

            d = r.json()
            ch = d.get("choices", [{}])[0]
            msg = ch.get("message", {})
            finish_reason = ch.get("finish_reason")
            tool_calls = msg.get("tool_calls") or []
            content = msg.get("content") or ""

            trial_ok = finish_reason in ("stop", "length") and len(tool_calls) == 0
            if trial_ok:
                ok += 1
            if tool_calls:
                false_tool_calls += 1
            if looks_loop_like(content):
                loop_like += 1

            if args.verbose:
                samples.append(
                    {
                        "trial": i + 1,
                        "ok": trial_ok,
                        "finish_reason": finish_reason,
                        "tool_calls": len(tool_calls),
                        "content_preview": str(content).replace("\n", " ")[:120],
                    }
                )
        except Exception as e:
            http_fail += 1
            if args.verbose:
                samples.append({"trial": i + 1, "exception": str(e)[:180]})

    return {
        "trials": args.plain_trials,
        "ok": ok,
        "http_fail": http_fail,
        "false_tool_calls": false_tool_calls,
        "loop_like": loop_like,
        "latencies": latencies,
        "samples": samples,
    }


def summarize_latencies(latencies: List[float]) -> Dict[str, float]:
    if not latencies:
        return {"p50_s": -1.0, "p95_s": -1.0, "mean_s": -1.0}
    sorted_l = sorted(latencies)
    p50 = sorted_l[int(0.50 * (len(sorted_l) - 1))]
    p95 = sorted_l[int(0.95 * (len(sorted_l) - 1))]
    return {"p50_s": p50, "p95_s": p95, "mean_s": statistics.mean(latencies)}


def main() -> None:
    args = parse_args()
    req = run_required_trials(args)
    plain = run_plain_trials(args)

    req_lat = summarize_latencies(req["latencies"])
    plain_lat = summarize_latencies(plain["latencies"])

    report = {
        "base_url": args.base_url,
        "model": args.model,
        "required_mode": {
            "success_rate": req["ok"] / max(req["trials"], 1),
            "ok": req["ok"],
            "trials": req["trials"],
            "http_fail": req["http_fail"],
            "malformed_args": req["malformed_args"],
            "loop_like_outputs": req["loop_like"],
            "latency": req_lat,
        },
        "plain_mode": {
            "success_rate": plain["ok"] / max(plain["trials"], 1),
            "ok": plain["ok"],
            "trials": plain["trials"],
            "http_fail": plain["http_fail"],
            "false_tool_calls": plain["false_tool_calls"],
            "loop_like_outputs": plain["loop_like"],
            "latency": plain_lat,
        },
    }

    print(json.dumps(report, indent=2))
    if args.verbose:
        print("\nrequired_samples:")
        print(json.dumps(req["samples"], indent=2))
        print("\nplain_samples:")
        print(json.dumps(plain["samples"], indent=2))


if __name__ == "__main__":
    main()
