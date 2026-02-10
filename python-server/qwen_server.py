"""
Qwen3-Coder llama-cpp-python OpenAI-compatible server with tool calling support

Serves Cerebras-Qwen3-Coder model on /v1/chat/completions endpoint
with automatic conversion of XML tool calls to OpenAI JSON format.
"""

import logging
import sys
import json
from contextlib import asynccontextmanager
from typing import List, Optional

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field

from llama_cpp import Llama

import config
from tool_parser import parse_tool_calls, extract_tool_calls_and_text, has_tool_calls

# Configure logging
logging.basicConfig(
    level=getattr(logging, config.LOG_LEVEL),
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


# Pydantic models for OpenAI API compatibility
class ToolFunction(BaseModel):
    name: str
    description: Optional[str] = None
    parameters: Optional[dict] = None


class Tool(BaseModel):
    type: str = "function"
    function: ToolFunction


class ChatMessage(BaseModel):
    role: str
    content: str
    tool_calls: Optional[list] = None


class ChatCompletionRequest(BaseModel):
    model: str
    messages: List[dict]
    temperature: float = Field(default=config.TEMPERATURE, ge=0, le=2.0)
    top_p: float = Field(default=config.TOP_P, ge=0, le=1.0)
    top_k: int = Field(default=config.TOP_K, ge=0)
    max_tokens: int = Field(default=config.MAX_TOKENS, ge=1)
    tools: Optional[List[Tool]] = None
    tool_choice: Optional[str] = None
    stream: bool = False
    repeat_penalty: float = Field(default=config.REPEAT_PENALTY, ge=0)
    presence_penalty: float = Field(default=config.PRESENCE_PENALTY, ge=-2.0, le=2.0)
    frequency_penalty: float = Field(default=config.FREQUENCY_PENALTY, ge=-2.0, le=2.0)


class ChatCompletionChoice(BaseModel):
    index: int
    message: dict
    finish_reason: str


class ChatCompletionResponse(BaseModel):
    id: str = "chatcmpl-qwen3"
    object: str = "chat.completion"
    created: int
    model: str
    choices: List[ChatCompletionChoice]
    usage: dict


class Usage(BaseModel):
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int


class HealthResponse(BaseModel):
    status: str
    model: str
    context_window: int
    tool_calling_enabled: bool


# Global model instance
llm: Optional[Llama] = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """FastAPI lifespan context manager for startup/shutdown"""
    global llm

    logger.info("=" * 60)
    logger.info("Qwen3-Coder llama-cpp-python Server Starting")
    logger.info("=" * 60)

    # Validate configuration
    if not config.validate_config():
        logger.error("Configuration validation failed!")
        sys.exit(1)

    # Load model
    try:
        logger.info(f"Loading model: {config.MODEL_NAME}")
        logger.info(f"Model path: {config.get_model_path()}")

        llm = Llama(
            model_path=config.get_model_path(),
            n_gpu_layers=config.N_GPU_LAYERS,
            n_ctx=config.N_CTX,
            n_batch=config.N_BATCH,
            n_ubatch=config.N_UBATCH,
            n_threads=config.N_THREADS,
            flash_attn=config.FLASH_ATTN,
            mlock=config.MLOCK,
            use_mmap=not config.NO_MMAP,
            type_k=config.CACHE_TYPE_K,
            type_v=config.CACHE_TYPE_V,
            use_jinja=config.USE_JINJA,
            verbose=config.VERBOSE,
        )

        logger.info(f"✓ Model loaded successfully")
        logger.info(f"✓ Context window: {config.N_CTX} tokens")
        logger.info(f"✓ GPU layers: {config.N_GPU_LAYERS}")
        logger.info(f"✓ Tool calling: {'ENABLED' if config.ENABLE_TOOL_CALLING else 'DISABLED'}")
        logger.info(f"✓ Server URL: {config.get_server_url()}")
        logger.info("=" * 60)

    except Exception as e:
        logger.error(f"Failed to load model: {e}")
        sys.exit(1)

    yield  # Server runs here

    # Cleanup
    logger.info("Server shutting down...")
    if llm:
        del llm
    logger.info("✓ Cleanup complete")


app = FastAPI(title="Qwen3-Coder Server", lifespan=lifespan)

# CORS middleware (Add this for OpenCode/Browser compatibility)
# REVERT: Remove this block if OpenCode still fails or if security is a concern
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
async def health_check() -> HealthResponse:
    """Health check endpoint"""
    if llm is None:
        raise HTTPException(status_code=503, detail="Model not loaded")

    return HealthResponse(
        status="ok",
        model=config.MODEL_ID,
        context_window=config.N_CTX,
        tool_calling_enabled=config.ENABLE_TOOL_CALLING,
    )


@app.get("/v1/models")
async def list_models():
    """List available models (OpenAI compatibility)"""
    return {
        "object": "list",
        "data": [
            {
                "id": config.MODEL_ID,
                "object": "model",
                "owned_by": "cerebras",
                "permission": [],
            }
        ],
    }


async def stream_chat_completion(response_generator):
    """Stream chat completion chunks in SSE format with tool call handling"""
    try:
        content_buffer = ""
        yielded_text_length = 0
        chunk_id = None
        chunk_model = None
        chunk_created = None
        tool_call_detected = False
        tokens_since_tool_detected = 0
        finished_with_tool_calls = False

        for chunk in response_generator:
            # Store chunk metadata
            if not chunk_id:
                chunk_id = chunk.get("id")
                chunk_model = chunk.get("model")
            chunk_created = chunk.get("created", chunk_created)

            # Check if this chunk has content
            if "choices" in chunk and len(chunk["choices"]) > 0:
                choice = chunk["choices"][0]
                delta = choice.get("delta", {})
                finish_reason = choice.get("finish_reason")

                # Buffer content
                if "content" in delta and delta["content"]:
                    content_buffer += delta["content"]

                    if tool_call_detected:
                        tokens_since_tool_detected += 1
                        if tokens_since_tool_detected > 500:
                            logger.warning("Watchdog: Runaway generation detected after tool call. Forcing flush.")
                            break

                    # SAFEGUARD: If model outputs stop tokens, stop immediately
                    stop_indicators = ["<|im_start|>", "<|im_end|>", "<|endoftext|>"]
                    if any(indicator in content_buffer[yielded_text_length:] for indicator in stop_indicators):
                        logger.info("Natural stop sequence detected in buffer.")
                        break

                # Check for tool call trigger
                if not tool_call_detected:
                    if "<tool_call>" in content_buffer or "<function=" in content_buffer:
                        tool_call_detected = True
                        tokens_since_tool_detected = 0
                        # Yield any text that appeared BEFORE the tool call
                        trigger = "<tool_call>" if "<tool_call>" in content_buffer else "<function="
                        text_before = content_buffer[:content_buffer.find(trigger)]
                        if len(text_before) > yielded_text_length:
                            text_to_yield = text_before[yielded_text_length:]
                            if text_to_yield:
                                temp_chunk = dict(chunk)
                                temp_chunk["choices"][0]["delta"] = {"content": text_to_yield}
                                yield f"data: {json.dumps(temp_chunk)}\n\n"
                                yielded_text_length = len(text_before)
                    else:
                        # Yield regular text, hold back partial '<' to avoid splitting tags
                        last_lt = content_buffer.rfind("<")
                        if last_lt > yielded_text_length:
                            text_to_yield = content_buffer[yielded_text_length:last_lt]
                            if text_to_yield:
                                temp_chunk = dict(chunk)
                                temp_chunk["choices"][0]["delta"] = {"content": text_to_yield}
                                yield f"data: {json.dumps(temp_chunk)}\n\n"
                                yielded_text_length = last_lt
                        elif last_lt == -1:
                            text_to_yield = content_buffer[yielded_text_length:]
                            if text_to_yield:
                                yield f"data: {json.dumps(chunk)}\n\n"
                                yielded_text_length = len(content_buffer)

                # Process complete tool call
                if tool_call_detected:
                    if "</tool_call>" in content_buffer or "</function>" in content_buffer:
                        has_calls, tool_calls = parse_tool_calls(content_buffer)
                        if has_calls and tool_calls:
                            for idx, tc in enumerate(tool_calls):
                                tool_chunk = {
                                    "id": chunk_id,
                                    "object": "chat.completion.chunk",
                                    "created": chunk_created,
                                    "model": chunk_model,
                                    "choices": [{
                                        "index": 0,
                                        "delta": {
                                            "tool_calls": [{
                                                "index": idx,
                                                "id": tc.get("id", f"call_{idx}"),
                                                "type": "function",
                                                "function": {
                                                    "name": tc["function"]["name"],
                                                    "arguments": tc["function"]["arguments"]
                                                }
                                            }]
                                        },
                                        "finish_reason": None
                                    }]
                                }
                                yield f"data: {json.dumps(tool_chunk)}\n\n"

                            # Send final chunk with tool_calls finish reason
                            finish_chunk = {
                                "id": chunk_id,
                                "object": "chat.completion.chunk",
                                "created": chunk_created,
                                "model": chunk_model,
                                "choices": [{
                                    "index": 0,
                                    "delta": {},
                                    "finish_reason": "tool_calls"
                                }]
                            }
                            yield f"data: {json.dumps(finish_chunk)}\n\n"
                            finished_with_tool_calls = True
                            break

                # If we get a natural finish reason from the engine, respect it
                if finish_reason:
                    break
            else:
                # Pass through non-content chunks
                yield f"data: {json.dumps(chunk)}\n\n"

        # FLUSH REMAINING BUFFER (only if we didn't already finish with tool_calls)
        if not finished_with_tool_calls and len(content_buffer) > yielded_text_length:
            remaining_text = content_buffer[yielded_text_length:]

            # If we were in a tool call but it never closed, try to parse anyway
            if tool_call_detected:
                has_calls, tool_calls = parse_tool_calls(content_buffer)
                if has_calls and tool_calls:
                    for idx, tc in enumerate(tool_calls):
                        tool_chunk = {
                            "id": chunk_id,
                            "object": "chat.completion.chunk",
                            "created": chunk_created,
                            "model": chunk_model,
                            "choices": [{
                                "index": 0,
                                "delta": {
                                    "tool_calls": [{
                                        "index": idx,
                                        "id": tc.get("id", f"call_flush_{idx}"),
                                        "type": "function",
                                        "function": {
                                            "name": tc["function"]["name"],
                                            "arguments": tc["function"]["arguments"]
                                        }
                                    }]
                                },
                                "finish_reason": None
                            }]
                        }
                        yield f"data: {json.dumps(tool_chunk)}\n\n"
                    finished_with_tool_calls = True
                    remaining_text = ""

            # Yield any remaining plain text
            if remaining_text:
                for stop_seq in ["<|im_end|>", "<|endoftext|>", "<|im_start|>"]:
                    remaining_text = remaining_text.split(stop_seq)[0]

                if remaining_text.strip():
                    flush_chunk = {
                        "id": chunk_id,
                        "object": "chat.completion.chunk",
                        "created": chunk_created,
                        "model": chunk_model,
                        "choices": [{
                            "index": 0,
                            "delta": {"content": remaining_text},
                            "finish_reason": None
                        }]
                    }
                    yield f"data: {json.dumps(flush_chunk)}\n\n"

        # Send exactly ONE final chunk with the correct finish_reason
        final_reason = "tool_calls" if finished_with_tool_calls else "stop"
        final_chunk = {
            "id": chunk_id,
            "object": "chat.completion.chunk",
            "created": chunk_created,
            "model": chunk_model,
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": final_reason
            }]
        }
        # Only send the stop chunk if we didn't already send a tool_calls finish
        if not finished_with_tool_calls:
            yield f"data: {json.dumps(final_chunk)}\n\n"
        yield "data: [DONE]\n\n"

    except Exception as e:
        logger.error(f"Error in streaming: {e}")
        error_chunk = {
            "error": {
                "message": str(e),
                "type": "stream_error"
            }
        }
        yield f"data: {json.dumps(error_chunk)}\n\n"


def _format_tools_for_prompt(tools_list: list) -> str:
    """
    Format tool definitions into a clear prompt format so the model
    understands what tools are available and their required parameters.
    """
    if not tools_list:
        return ""

    prompt = "You have access to the following tools:\n\n"

    for tool in tools_list:
        if tool.get("type") != "function":
            continue

        func = tool.get("function", {})
        name = func.get("name", "unknown")
        desc = func.get("description", "No description")
        params = func.get("parameters", {})

        prompt += f"<tool name=\"{name}\">\n"
        prompt += f"  description: {desc}\n"

        # Extract parameters with required fields
        props = params.get("properties", {})
        required = params.get("required", [])

        if props:
            prompt += "  parameters:\n"
            for param_name, param_def in props.items():
                param_type = param_def.get("type", "string")
                param_desc = param_def.get("description", "")
                is_required = param_name in required
                req_str = " [REQUIRED]" if is_required else " [OPTIONAL]"
                prompt += f"    - {param_name}: {param_type}{req_str}"
                if param_desc:
                    prompt += f" - {param_desc}"
                prompt += "\n"

        prompt += "</tool>\n\n"

    prompt += "When using tools, provide all REQUIRED parameters. Use XML format: <tool_call><function=name><parameter=param_name>value</parameter></function></tool_call>\n"
    return prompt


@app.post("/v1/chat/completions")
async def chat_completions(request: ChatCompletionRequest):
    """OpenAI-compatible chat completions endpoint"""
    if llm is None:
        raise HTTPException(status_code=503, detail="Model not loaded")

    try:
        # Prepare messages for the model
        messages = request.messages
        logger.debug(f"Messages: {json.dumps(messages, indent=2)}")
        if request.tools:
            logger.debug(f"Tools: {json.dumps([t.model_dump() for t in request.tools], indent=2)}")

        # Convert OpenAI format tool_calls (arguments as JSON string) to llama-cpp format (arguments as dict)
        # This is needed because llama-cpp's chat template expects dict, not string
        for message in messages:
            if isinstance(message, dict) and message.get("role") == "assistant" and "tool_calls" in message:
                for tool_call in message.get("tool_calls", []):
                    if "function" in tool_call and "arguments" in tool_call["function"]:
                        args = tool_call["function"]["arguments"]
                        # If arguments is a string, parse it to dict
                        if isinstance(args, str):
                            try:
                                tool_call["function"]["arguments"] = json.loads(args)
                            except json.JSONDecodeError:
                                logger.warning(f"Failed to parse tool call arguments: {args}")

        # Add tools to system message if provided
        tools_list = None
        has_tools = False
        if request.tools and config.ENABLE_TOOL_CALLING:
            tools_list = [t.model_dump() for t in request.tools]
            has_tools = True

            # Inject tool definitions into system message for better model understanding
            tools_prompt = _format_tools_for_prompt(tools_list)

            # Find or create system message
            system_msg_idx = -1
            for i, msg in enumerate(messages):
                if msg.get("role") == "system":
                    system_msg_idx = i
                    break

            if system_msg_idx >= 0:
                # Append tool definitions to existing system message
                messages[system_msg_idx]["content"] += "\n\n" + tools_prompt
            else:
                # Create new system message with tool definitions
                messages.insert(0, {"role": "system", "content": tools_prompt})


        # Get completion from model
        logger.debug(f"Processing request with {len(messages)} messages, tools={has_tools}, stream={request.stream}")

        # Explicit stop tokens for Qwen3-Coder/ChatML
        stop_tokens = [
            "<|im_end|>", 
            "<|endoftext|>", 
            "<|im_start|>user", 
            "<|im_start|>system",
            "\n<|im_start|>",
            "\n<|im_end|>"
        ]

        # Force stop logic and generation limits
        gen_params = {
            "messages": messages,
            "temperature": request.temperature,
            "top_p": request.top_p,
            "top_k": request.top_k,
            "max_tokens": min(request.max_tokens, 16384),
            "repeat_penalty": request.repeat_penalty,
            "presence_penalty": request.presence_penalty,
            "frequency_penalty": request.frequency_penalty,
            "tools": tools_list,
            "stream": request.stream,
            "stop": stop_tokens,
        }

        response = llm.create_chat_completion(**gen_params)

        # Handle streaming vs non-streaming responses
        if request.stream:
            # Return streaming response with SSE format (no tools present)
            return StreamingResponse(
                stream_chat_completion(response),
                media_type="text/event-stream",
                headers={
                    "Cache-Control": "no-cache",
                    "Connection": "keep-alive",
                }
            )
        else:
            # Non-streaming response - process tool calls if present
            response = _process_tool_calls(response)
            logger.debug(f"Final response: {json.dumps(response, indent=2)}")
            return response

    except Exception as e:
        logger.error(f"Error processing request: {e}")
        raise HTTPException(status_code=500, detail=str(e))


def _process_tool_calls(response: dict) -> dict:
    """
    Post-process response to convert XML tool calls to OpenAI JSON format.

    The llama-cpp-python library may return tool calls in the model's native
    XML format. This function parses them and converts to OpenAI format.
    """
    if not config.ENABLE_TOOL_CALLING or "choices" not in response:
        return response

    for choice in response.get("choices", []):
        if "message" not in choice:
            continue

        message = choice["message"]
        content = message.get("content", "")

        # Check if response contains tool calls (wrapped or unwrapped)
        if "<tool_call>" in content or "<function=" in content:
            # Parse XML tool calls
            has_calls, tool_calls = parse_tool_calls(content)

            if has_calls and tool_calls:
                logger.info(f"Parsed {len(tool_calls)} tool calls from model output")

                # Extract text before tool calls
                text, _ = extract_tool_calls_and_text(content)

                # Update message with parsed tool calls
                # IMPORTANT: If text is empty or just whitespace, set content to None
                # to match OpenAI convention for tool call only responses
                message["content"] = text.strip() if text and text.strip() else None
                message["tool_calls"] = tool_calls

                # Set finish reason to 'tool_calls'
                choice["finish_reason"] = "tool_calls"

                logger.debug(f"Tool calls: {json.dumps(tool_calls, indent=2)}")

    return response


@app.post("/v1/completions")
async def completions(request: dict):
    """OpenAI-compatible text completions endpoint (fallback)"""
    if llm is None:
        raise HTTPException(status_code=503, detail="Model not loaded")

    raise HTTPException(
        status_code=400, detail="Text completions not supported. Use /v1/chat/completions"
    )


@app.get("/")
async def root():
    """Root endpoint with server info"""
    return {
        "status": "running",
        "name": "Qwen3-Coder Server",
        "model": config.MODEL_NAME,
        "version": "1.0.0",
        "endpoints": {
            "health": "/health",
            "chat": "/v1/chat/completions",
            "models": "/v1/models",
        },
    }


if __name__ == "__main__":
    import uvicorn

    logger.info(f"Starting server on {config.SERVER_HOST}:{config.SERVER_PORT}")

    uvicorn.run(
        app,
        host=config.SERVER_HOST,
        port=config.SERVER_PORT,
        workers=config.SERVER_WORKERS,
        log_level=config.LOG_LEVEL.lower(),
    )
