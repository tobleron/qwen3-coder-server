# API Documentation

Complete API reference for the Qwen3-Coder Server.

## Base URL

```
http://localhost:8081/v1
```

For remote access, replace `localhost` with your server's IP address.

## Authentication

This server does **not** require authentication. No API key needed.

## Endpoints

### 1. Health Check

Check if server is running and model is loaded.

**Endpoint:** `GET /health`

**Example:**
```bash
curl http://localhost:8081/health
```

**Response (200 OK):**
```json
{
  "status": "ok",
  "model": "cerebras-qwen3",
  "context_window": 131072,
  "tool_calling_enabled": true
}
```

---

### 2. List Models

Get available models.

**Endpoint:** `GET /v1/models`

**Example:**
```bash
curl http://localhost:8081/v1/models
```

**Response (200 OK):**
```json
{
  "object": "list",
  "data": [
    {
      "id": "cerebras-qwen3",
      "object": "model",
      "owned_by": "cerebras",
      "permission": []
    }
  ]
}
```

---

### 3. Chat Completions (Main Endpoint)

Generate chat completions with optional tool calling.

**Endpoint:** `POST /v1/chat/completions`

**Request Headers:**
```
Content-Type: application/json
```

**Request Body:**

```json
{
  "model": "cerebras-qwen3",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful coding assistant."
    },
    {
      "role": "user",
      "content": "Write a Python function to sort a list."
    }
  ],
  "temperature": 0.7,
  "top_p": 0.8,
  "top_k": 20,
  "max_tokens": 1000,
  "repeat_penalty": 1.1,
  "tools": [],
  "stream": false
}
```

**Request Parameters:**

| Parameter | Type | Required | Default | Range | Description |
|-----------|------|----------|---------|-------|-------------|
| `model` | string | Yes | - | - | Model ID (always "cerebras-qwen3") |
| `messages` | array | Yes | - | - | Chat message history |
| `temperature` | float | No | 0.7 | 0.0-2.0 | Response randomness |
| `top_p` | float | No | 0.8 | 0.0-1.0 | Nucleus sampling |
| `top_k` | int | No | 20 | 1-100 | Top-K sampling |
| `max_tokens` | int | No | 16384 | 1-16384 | Max output tokens |
| `repeat_penalty` | float | No | 1.1 | 0.0+ | Repetition penalty |
| `frequency_penalty` | float | No | 0.0 | -2.0-2.0 | Frequency penalty |
| `presence_penalty` | float | No | 0.0 | -2.0-2.0 | Presence penalty |
| `tools` | array | No | [] | - | Tool definitions (see below) |
| `tool_choice` | string | No | - | - | Force tool use ("auto", "required") |
| `stream` | bool | No | false | - | Stream response chunks |

**Message Format:**

```json
{
  "role": "system|user|assistant|tool",
  "content": "message text"
}
```

For assistant messages with tool calls:
```json
{
  "role": "assistant",
  "content": null,
  "tool_calls": [
    {
      "id": "call_abc123",
      "type": "function",
      "function": {
        "name": "bash",
        "arguments": "{\"command\": \"ls -la\"}"
      }
    }
  ]
}
```

For tool response:
```json
{
  "role": "tool",
  "content": "bash command output here",
  "tool_call_id": "call_abc123"
}
```

**Tool Definition Format:**

```json
{
  "type": "function",
  "function": {
    "name": "bash",
    "description": "Execute bash commands on the system",
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
```

**Response (200 OK) - Non-streaming:**

```json
{
  "id": "chatcmpl-qwen3",
  "object": "chat.completion",
  "created": 1707531234,
  "model": "cerebras-qwen3",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Here's a Python sorting function:\n\n```python\ndef sort_list(items):\n    return sorted(items)\n```"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 45,
    "completion_tokens": 28,
    "total_tokens": 73
  }
}
```

**Response (200 OK) - With Tool Calls:**

```json
{
  "id": "chatcmpl-qwen3",
  "object": "chat.completion",
  "created": 1707531234,
  "model": "cerebras-qwen3",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": null,
        "tool_calls": [
          {
            "id": "call_abc123",
            "type": "function",
            "function": {
              "name": "bash",
              "arguments": "{\"command\": \"python --version\"}"
            }
          }
        ]
      },
      "finish_reason": "tool_calls"
    }
  ],
  "usage": {
    "prompt_tokens": 120,
    "completion_tokens": 15,
    "total_tokens": 135
  }
}
```

**Streaming Response (200 OK, chunked):**

When `stream=true`, response is Server-Sent Events (SSE):

```
data: {"id":"chatcmpl-qwen3","object":"chat.completion.chunk","created":1707531234,"model":"cerebras-qwen3","choices":[{"index":0,"delta":{"content":"Here"},"finish_reason":null}]}

data: {"id":"chatcmpl-qwen3","object":"chat.completion.chunk","created":1707531234,"model":"cerebras-qwen3","choices":[{"index":0,"delta":{"content":" is"},"finish_reason":null}]}

data: [DONE]
```

**Error Responses:**

| Status | Error | Description |
|--------|-------|-------------|
| 400 | Bad Request | Invalid request format |
| 503 | Service Unavailable | Model not loaded |
| 500 | Internal Server Error | Server error (check logs) |

Example error:
```json
{
  "detail": "Model not loaded"
}
```

---

## Usage Examples

### Example 1: Simple Chat

```bash
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "cerebras-qwen3",
    "messages": [
      {"role": "user", "content": "What is Python?"}
    ],
    "max_tokens": 200,
    "temperature": 0.7
  }'
```

### Example 2: Chat with Tool Calling

```bash
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "cerebras-qwen3",
    "messages": [
      {"role": "user", "content": "List the files in the current directory using bash"}
    ],
    "tools": [
      {
        "type": "function",
        "function": {
          "name": "bash",
          "description": "Execute bash commands",
          "parameters": {
            "type": "object",
            "properties": {
              "command": {"type": "string", "description": "Bash command"}
            },
            "required": ["command"]
          }
        }
      }
    ],
    "max_tokens": 500
  }'
```

### Example 3: Multi-turn Conversation with Tools

```bash
# First request
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "cerebras-qwen3",
    "messages": [
      {"role": "user", "content": "Check the Python version"}
    ],
    "tools": [
      {
        "type": "function",
        "function": {
          "name": "bash",
          "description": "Execute bash commands",
          "parameters": {
            "type": "object",
            "properties": {
              "command": {"type": "string"}
            },
            "required": ["command"]
          }
        }
      }
    ]
  }'

# Response includes: finish_reason="tool_calls" and tool_calls array
# Extract the tool call and execute it: python --version
# Response: Python 3.11.2

# Second request - send tool result back
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "cerebras-qwen3",
    "messages": [
      {"role": "user", "content": "Check the Python version"},
      {
        "role": "assistant",
        "content": null,
        "tool_calls": [
          {
            "id": "call_abc123",
            "type": "function",
            "function": {
              "name": "bash",
              "arguments": "{\"command\": \"python --version\"}"
            }
          }
        ]
      },
      {
        "role": "tool",
        "content": "Python 3.11.2",
        "tool_call_id": "call_abc123"
      }
    ]
  }'

# Model responds based on tool result
```

### Example 4: Streaming Response

```bash
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "cerebras-qwen3",
    "messages": [{"role": "user", "content": "Write a poem"}],
    "stream": true,
    "max_tokens": 200
  }' \
  | grep -oP '"content":"\K[^"]*' | tr -d '\n' && echo
```

### Example 5: Custom Parameters

```bash
curl -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "cerebras-qwen3",
    "messages": [{"role": "user", "content": "Hello"}],
    "temperature": 0.5,
    "top_p": 0.9,
    "top_k": 40,
    "max_tokens": 500,
    "repeat_penalty": 1.05,
    "frequency_penalty": 0.0,
    "presence_penalty": 0.0
  }'
```

## Python Client Example

Using `openai` Python library (compatible):

```python
from openai import OpenAI

client = OpenAI(
    api_key="sk-no-key-required",
    base_url="http://localhost:8081/v1"
)

# Simple chat
response = client.chat.completions.create(
    model="cerebras-qwen3",
    messages=[
        {"role": "user", "content": "What is Python?"}
    ],
    max_tokens=500,
    temperature=0.7
)

print(response.choices[0].message.content)

# With tools
tools = [
    {
        "type": "function",
        "function": {
            "name": "bash",
            "description": "Execute bash commands",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                },
                "required": ["command"]
            }
        }
    }
]

response = client.chat.completions.create(
    model="cerebras-qwen3",
    messages=[
        {"role": "user", "content": "List files"}
    ],
    tools=tools,
    max_tokens=500
)

if response.choices[0].finish_reason == "tool_calls":
    for tool_call in response.choices[0].message.tool_calls:
        print(f"Called: {tool_call.function.name}")
        print(f"Args: {tool_call.function.arguments}")
```

## Performance Notes

### Throughput

- **Prompt processing:** 75-125 tokens/sec
- **Generation:** 2-3 tokens/sec
- **First request:** +30-40 seconds (model loading)

### Latency

- **Simple queries (50-100 tokens):** 5-10 seconds
- **Complex queries (500 tokens):** 10-20 seconds
- **Tool calling:** Same latency, but finish_reason changes

### Memory

- **Peak VRAM:** 11.5 GB on RTX 3060
- **Context:** 131,072 tokens (configurable)
- **Batch:** 512 tokens (configurable)

## Rate Limiting

There is **no built-in rate limiting**. The server processes requests sequentially.

## Error Handling

Always check response status codes:

```python
import requests

response = requests.post(
    "http://localhost:8081/v1/chat/completions",
    json={...}
)

if response.status_code == 200:
    data = response.json()
    # Process response
elif response.status_code == 503:
    print("Model not loaded")
elif response.status_code == 500:
    print(f"Server error: {response.text}")
```

## Best Practices

1. **Reuse connection:** Keep HTTP connection open for multiple requests
2. **Set reasonable timeouts:** 60+ seconds for generation
3. **Handle tool calls:** If `finish_reason="tool_calls"`, execute tools and send results back
4. **Monitor logs:** Check `python_server.log` for errors
5. **Use streaming:** For UI responsiveness (tokens appear as generated)
6. **Context management:** Track total tokens to stay within 131k limit

---

**Last Updated:** February 2026
