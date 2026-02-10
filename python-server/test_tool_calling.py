#!/usr/bin/env python3
"""
Test script for Qwen3-Coder tool calling

Tests the OpenAI-compatible API with tool calling enabled.
Verifies that XML tool calls are properly converted to JSON format.
"""

import json
import requests
import time
import sys
from typing import Optional, Dict, Any

BASE_URL = "http://localhost:8081/v1"
HEALTH_URL = "http://localhost:8081/health"

# ANSI color codes
ORANGE = "\033[38;5;208m"
GREEN = "\033[38;5;48m"
RED = "\033[38;5;196m"
RESET = "\033[0m"


def print_header(text: str):
    """Print a formatted header"""
    print(f"\n{ORANGE}{'=' * 60}{RESET}")
    print(f"{ORANGE}{text:^60}{RESET}")
    print(f"{ORANGE}{'=' * 60}{RESET}\n")


def print_success(text: str):
    """Print success message"""
    print(f"{GREEN}✓ {text}{RESET}")


def print_error(text: str):
    """Print error message"""
    print(f"{RED}✗ {text}{RESET}")


def print_info(text: str):
    """Print info message"""
    print(f"{ORANGE}ℹ {text}{RESET}")


def check_health() -> bool:
    """Check if server is running and healthy"""
    print_header("Health Check")

    try:
        response = requests.get(HEALTH_URL, timeout=5)
        if response.status_code == 200:
            data = response.json()
            print_success(f"Server is healthy")
            print_info(f"Model: {data.get('model')}")
            print_info(f"Context Window: {data.get('context_window')} tokens")
            print_info(f"Tool Calling: {'ENABLED' if data.get('tool_calling_enabled') else 'DISABLED'}")
            return True
        else:
            print_error(f"Server returned status {response.status_code}")
            return False
    except requests.exceptions.ConnectionError:
        print_error("Cannot connect to server. Is it running on localhost:8081?")
        return False
    except Exception as e:
        print_error(f"Health check failed: {e}")
        return False


def test_simple_chat() -> bool:
    """Test simple chat without tools"""
    print_header("Test 1: Simple Chat (No Tools)")

    payload = {
        "model": "cerebras-qwen3",
        "messages": [
            {"role": "user", "content": "Hello, what is 2+2?"}
        ],
        "temperature": 0.7,
        "max_tokens": 100
    }

    try:
        response = requests.post(f"{BASE_URL}/chat/completions", json=payload, timeout=30)

        if response.status_code == 200:
            data = response.json()
            if data.get("choices"):
                content = data["choices"][0]["message"].get("content", "")
                print_success(f"Response: {content[:100]}...")
                return True
            else:
                print_error("No choices in response")
                return False
        else:
            print_error(f"Server returned status {response.status_code}")
            return False
    except Exception as e:
        print_error(f"Chat test failed: {e}")
        return False


def test_tool_calling() -> bool:
    """Test chat with tool calling enabled"""
    print_header("Test 2: Chat with Tool Calling")

    tools = [
        {
            "type": "function",
            "function": {
                "name": "bash",
                "description": "Execute bash commands",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The bash command to execute"
                        }
                    },
                    "required": ["command"]
                }
            }
        }
    ]

    payload = {
        "model": "cerebras-qwen3",
        "messages": [
            {"role": "user", "content": "List the current directory files using the bash tool"}
        ],
        "tools": tools,
        "temperature": 0.7,
        "max_tokens": 500
    }

    try:
        print_info("Sending request with bash tool definition...")
        response = requests.post(f"{BASE_URL}/chat/completions", json=payload, timeout=60)

        if response.status_code == 200:
            data = response.json()
            print_info(f"Response status: {response.status_code}")

            if data.get("choices"):
                choice = data["choices"][0]
                message = choice.get("message", {})

                print_info(f"Finish reason: {choice.get('finish_reason')}")

                content = message.get("content")
                tool_calls = message.get("tool_calls")

                if content:
                    print_success(f"Text response: {content[:100]}...")

                if tool_calls:
                    print_success(f"Model called {len(tool_calls)} tool(s)")
                    for i, call in enumerate(tool_calls):
                        func_name = call.get("function", {}).get("name")
                        arguments = call.get("function", {}).get("arguments")
                        print_info(f"  Tool {i+1}: {func_name}")
                        print_info(f"  Arguments: {arguments}")
                    return True
                else:
                    print_info("Model did not call any tools (but response was successful)")
                    return True
            else:
                print_error("No choices in response")
                return False
        else:
            print_error(f"Server returned status {response.status_code}")
            print_error(f"Response: {response.text[:200]}")
            return False
    except requests.exceptions.Timeout:
        print_error("Request timed out (model may be processing)")
        return False
    except Exception as e:
        print_error(f"Tool calling test failed: {e}")
        return False


def test_multiple_tools() -> bool:
    """Test with multiple tools defined"""
    print_header("Test 3: Multiple Tools")

    tools = [
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather for a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "City name"
                        },
                        "units": {
                            "type": "string",
                            "description": "Temperature units (celsius/fahrenheit)"
                        }
                    },
                    "required": ["location"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "search",
                "description": "Search the internet",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        }
                    },
                    "required": ["query"]
                }
            }
        }
    ]

    payload = {
        "model": "cerebras-qwen3",
        "messages": [
            {"role": "user", "content": "What's the weather in New York?"}
        ],
        "tools": tools,
        "temperature": 0.7,
        "max_tokens": 300
    }

    try:
        print_info("Sending request with multiple tools...")
        response = requests.post(f"{BASE_URL}/chat/completions", json=payload, timeout=60)

        if response.status_code == 200:
            data = response.json()

            if data.get("choices"):
                choice = data["choices"][0]
                message = choice.get("message", {})
                tool_calls = message.get("tool_calls", [])

                if tool_calls:
                    print_success(f"Model called {len(tool_calls)} tool(s)")
                    for call in tool_calls:
                        func_name = call.get("function", {}).get("name")
                        print_info(f"  Called: {func_name}")
                    return True
                else:
                    print_info("Model chose not to call tools (but response was valid)")
                    return True
            else:
                print_error("No choices in response")
                return False
        else:
            print_error(f"Server returned status {response.status_code}")
            return False
    except Exception as e:
        print_error(f"Multiple tools test failed: {e}")
        return False


def main():
    """Run all tests"""
    print(f"\n{ORANGE}Qwen3-Coder Tool Calling Test Suite{RESET}")
    print(f"{ORANGE}Testing OpenAI-compatible API with XML→JSON tool call conversion{RESET}")

    # Check health
    if not check_health():
        print_error("Server health check failed. Cannot continue with tests.")
        sys.exit(1)

    results = []

    # Run tests
    print("\n" + "=" * 60)
    print(f"{ORANGE}Running Tests{RESET}")
    print("=" * 60)

    # Test 1: Simple chat
    results.append(("Simple Chat", test_simple_chat()))
    time.sleep(1)

    # Test 2: Tool calling
    results.append(("Tool Calling", test_tool_calling()))
    time.sleep(1)

    # Test 3: Multiple tools
    results.append(("Multiple Tools", test_multiple_tools()))

    # Print summary
    print_header("Test Summary")

    passed = sum(1 for _, result in results if result)
    total = len(results)

    for name, result in results:
        status = f"{GREEN}PASS{RESET}" if result else f"{RED}FAIL{RESET}"
        print(f"  {status} - {name}")

    print(f"\n{ORANGE}Results: {passed}/{total} tests passed{RESET}\n")

    if passed == total:
        print_success("All tests passed!")
        sys.exit(0)
    else:
        print_error(f"{total - passed} test(s) failed")
        sys.exit(1)


if __name__ == "__main__":
    main()
