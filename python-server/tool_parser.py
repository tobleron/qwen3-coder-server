"""
Qwen3 Coder Tool Call Parser

Parses XML-formatted tool calls from Qwen3-Coder model output
and converts them to OpenAI-compatible JSON format.

Based on: https://huggingface.co/cerebras/Qwen3-Coder-REAP-25B-A3B
"""

import re
import json
import uuid
from typing import Dict, List, Any, Optional, Tuple


import logging
logger = logging.getLogger(__name__)

class Qwen3CoderToolParser:
    """Parse Qwen3-Coder tool calls from XML format to JSON"""

    def __init__(self):
        self.tool_call_start_token = "<tool_call>"
        self.tool_call_end_token = "</tool_call>"
        self.function_prefix = "<function="
        self.function_end_token = "</function>"
        self.parameter_prefix = "<parameter="
        self.parameter_end_token = "</parameter>"

        # Regex patterns for extraction
        self.tool_call_regex = re.compile(r"<tool_call>(.*?)</tool_call>", re.DOTALL)
        self.function_regex = re.compile(
            r"<function=([^>\n]+)>(.*?)(?:</function>|$)", re.DOTALL
        )
        self.parameter_regex = re.compile(
            r"<parameter=([^>]+)>(.*?)(?:</parameter>|(?=<parameter=)|(?=</function>)|$)",
            re.DOTALL,
        )

    def parse_tool_calls(
        self, model_output: str, tools: Optional[List[Dict[str, Any]]] = None
    ) -> Tuple[bool, List[Dict[str, Any]]]:
        """
        Extract explicit tool calls from model output.
        This parser intentionally avoids heuristic argument guessing because
        fabricated arguments are worse than a missed call for agent reliability.
        """
        if self.function_prefix not in model_output:
            return False, []

        tool_schemas = self._build_tool_schemas(tools)
        allowed_tool_names = set(tool_schemas.keys()) if tool_schemas else None

        tool_calls = []
        func_blocks = self._extract_function_blocks(model_output)

        for func_name, func_content in func_blocks:
            func_name = func_name.strip()
            if not func_name:
                continue

            if allowed_tool_names is not None and func_name not in allowed_tool_names:
                logger.warning(f"Ignoring unknown tool call: {func_name}")
                continue

            arguments: Dict[str, Any] = {}
            param_matches = list(self.parameter_regex.finditer(func_content))
            if param_matches:
                for param_match in param_matches:
                    param_name = param_match.group(1).strip()
                    param_value = (param_match.group(2) or "").strip()
                    if not param_name:
                        continue
                    arguments[param_name] = self._parse_argument_value(param_value)
            else:
                # Recovery path: allow unwrapped single-arg payload only when schema is unambiguous.
                raw_value = (func_content or "").strip()
                if raw_value and func_name in tool_schemas:
                    required = tool_schemas[func_name].get("required", [])
                    properties = tool_schemas[func_name].get("properties", {})
                    if len(required) == 1 and required[0] in properties:
                        arguments[required[0]] = self._parse_argument_value(raw_value)
                    else:
                        logger.warning(f"Skipping ambiguous tool call args for {func_name}")

            call = {
                "id": f"call_{uuid.uuid4().hex[:8]}",
                "type": "function",
                "function": {
                    "name": func_name,
                    "arguments": json.dumps(arguments)
                }
            }
            tool_calls.append(call)

        return len(tool_calls) > 0, tool_calls

    def _extract_function_blocks(self, model_output: str) -> List[Tuple[str, str]]:
        """
        Prefer functions inside <tool_call> blocks. Fallback to direct <function=...>
        only when wrapped blocks are absent.
        """
        blocks: List[Tuple[str, str]] = []

        wrapped_matches = list(self.tool_call_regex.finditer(model_output))
        if wrapped_matches:
            for match in wrapped_matches:
                block = match.group(1) or ""
                func_match = self.function_regex.search(block)
                if not func_match:
                    continue
                func_name = (func_match.group(1) or "").strip()
                func_content = func_match.group(2) or ""
                blocks.append((func_name, func_content))
            return blocks

        for func_match in self.function_regex.finditer(model_output):
            func_name = (func_match.group(1) or "").strip()
            func_content = func_match.group(2) or ""
            blocks.append((func_name, func_content))

        return blocks

    def _parse_argument_value(self, value: str) -> Any:
        value = value.strip()
        if not value:
            return ""

        if (value.startswith("[") and value.endswith("]")) or (
            value.startswith("{") and value.endswith("}")
        ):
            cleaned_value = value.strip("`").strip()
            try:
                return json.loads(cleaned_value)
            except json.JSONDecodeError:
                return value

        return value

    def _build_tool_schemas(
        self, tools: Optional[List[Dict[str, Any]]]
    ) -> Dict[str, Dict[str, Any]]:
        schemas: Dict[str, Dict[str, Any]] = {}
        if not tools:
            return schemas

        for tool in tools:
            if not isinstance(tool, dict):
                continue
            if tool.get("type") != "function":
                continue
            function = tool.get("function", {})
            if not isinstance(function, dict):
                continue
            name = function.get("name")
            parameters = function.get("parameters", {}) or {}
            if isinstance(name, str) and name:
                schemas[name] = {
                    "required": parameters.get("required", []) or [],
                    "properties": parameters.get("properties", {}) or {},
                }

        return schemas

    def _parse_tool_call_block(self, content: str) -> Optional[Dict[str, Any]]:
        """Parse a single <tool_call>...</tool_call> block"""
        func_match = self.function_regex.search(content)
        if not func_match:
            return None

        func_name = (func_match.group(1) or "").strip()
        if not func_name:
            return None

        func_content = func_match.group(2) or ""
        arguments: Dict[str, Any] = {}

        for param_match in self.parameter_regex.finditer(func_content):
            param_name = param_match.group(1).strip()
            param_value = (param_match.group(2) or "").strip()
            if not param_name:
                continue
            arguments[param_name] = self._parse_argument_value(param_value)

        # Return in OpenAI format
        return {
            "id": f"call_{uuid.uuid4().hex[:8]}",
            "type": "function",
            "function": {"name": func_name, "arguments": json.dumps(arguments)},
        }

    def extract_text_before_tool_call(self, model_output: str) -> str:
        """Extract any text that appears before the first tool call"""
        # Find the earliest occurrence of any tool-related tag
        indices = []
        for tag in [self.tool_call_start_token, self.function_prefix]:
            idx = model_output.find(tag)
            if idx != -1:
                indices.append(idx)
        
        if indices:
            return model_output[: min(indices)].strip()
        return model_output.strip()

    def has_tool_calls(self, text: str) -> bool:
        """Check if text contains tool calls"""
        return self.tool_call_start_token in text or self.function_prefix in text

    def parse_and_extract_text(
        self, model_output: str, tools: Optional[List[Dict[str, Any]]] = None
    ) -> Tuple[str, List[Dict[str, Any]]]:
        """
        Extract both the text response and tool calls from model output.

        Returns:
            Tuple of (text_response, tool_calls)
        """
        text = self.extract_text_before_tool_call(model_output)
        has_calls, tool_calls = self.parse_tool_calls(model_output, tools)
        return text, tool_calls


# Global parser instance
_parser = Qwen3CoderToolParser()


def parse_tool_calls(
    model_output: str, tools: Optional[List[Dict[str, Any]]] = None
) -> Tuple[bool, List[Dict[str, Any]]]:
    """Convenience function to parse tool calls"""
    return _parser.parse_tool_calls(model_output, tools)


def has_tool_calls(text: str) -> bool:
    """Convenience function to check for tool calls"""
    return _parser.has_tool_calls(text)


def extract_tool_calls_and_text(
    model_output: str, tools: Optional[List[Dict[str, Any]]] = None
) -> Tuple[str, List[Dict[str, Any]]]:
    """Convenience function to extract both text and tool calls"""
    return _parser.parse_and_extract_text(model_output, tools)
