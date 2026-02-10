"""
Qwen3 Coder Tool Call Parser

Parses XML-formatted tool calls from Qwen3-Coder model output
and converts them to OpenAI-compatible JSON format.

Based on: https://huggingface.co/cerebras/Qwen3-Coder-REAP-25B-A3B
"""

import re
import json
import ast
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
        self.tool_call_regex = re.compile(
            r"<tool_call>(.*?)</tool_call>|<tool_call>(.*?)$", re.DOTALL
        )
        self.function_regex = re.compile(
            r"<function=([^>]+)>(.*?)</function>|<function=([^>]+)>(.*)$", re.DOTALL
        )
        self.parameter_regex = re.compile(
            r"<parameter=([^>]+)>(.*?)(?:</parameter>|(?=<parameter=)|(?=</function>)|$)",
            re.DOTALL,
        )

    def parse_tool_calls(
        self, model_output: str, tools: Optional[List[Dict[str, Any]]] = None
    ) -> Tuple[bool, List[Dict[str, Any]]]:
        """
        Extract tool calls from model output with robust fallback for malformed tags.
        """
        if self.function_prefix not in model_output:
            return False, []

        logger.info(f"Parsing tool calls from raw output: {model_output}")
        tool_calls = []
        
        # Strategy: Find every <function=NAME> block
        func_blocks = re.findall(r"<function=([^>]+)>(.*?)(?:</function>|$)", model_output, re.DOTALL)
        
        for func_name, func_content in func_blocks:
            func_name = func_name.strip()
            arguments = {}
            
            # Extract parameters within this function block
            param_matches = self.parameter_regex.finditer(func_content)
            has_formal_params = False
            for param_match in param_matches:
                has_formal_params = True
                param_name = param_match.group(1).strip()
                param_value = (param_match.group(2) or "").strip()
                
                if param_name:
                    # Try to parse complex objects (like the 'todos' array)
                    if (param_value.startswith("[") and param_value.endswith("]")) or \
                       (param_value.startswith("{") and param_value.endswith("}")):
                        try:
                            cleaned = param_value.strip("`").strip()
                            arguments[param_name] = json.loads(cleaned)
                        except:
                            try:
                                arguments[param_name] = ast.literal_eval(param_value)
                            except:
                                arguments[param_name] = param_value
                    else:
                        arguments[param_name] = param_value
            
            # HEURISTIC FALLBACK: If no formal parameters found, try to extract from text
            if not has_formal_params:
                logger.info(f"Applying heuristics for {func_name} due to missing formal parameters")
                
                # Normalize tool name for heuristic matching
                norm_name = func_name.lower().replace("_", "")
                
                if norm_name in ["todowrite", "writetodos", "tasks"]:
                    # 1. Look for explicit JSON array [{}, {}]
                    array_match = re.search(r"(\[.*?\])", model_output, re.DOTALL)
                    if array_match:
                        try:
                            arguments["todos"] = json.loads(array_match.group(1).strip())
                            logger.info("Heuristically extracted 'todos' from JSON array in text")
                        except:
                            pass
                    
                    # 2. If no JSON, look for comma-separated lists or bullet points in the preamble
                    if "todos" not in arguments:
                        preamble = model_output[:model_output.find("<tool_call>")].strip()
                        items = []
                        
                        # Try to find list-like patterns: "alpha, bravo, and tango" or bullet points
                        # Split by "and", commas, or newlines with dashes/numbers
                        potential_items = re.split(r",\s*|\s+and\s+|\n\s*[\-\*•\d\.]+\s*", preamble)
                        for item in potential_items:
                            cleaned = item.strip().strip(".:;").strip()
                            if cleaned and len(cleaned) > 1 and not cleaned.lower().startswith("i'll create") and not cleaned.lower().startswith("here is"):
                                items.append({
                                    "id": f"task_{len(items)+1}",
                                    "content": cleaned,
                                    "status": "pending",
                                    "priority": "medium"
                                })
                        
                        if items:
                            arguments["todos"] = items
                            logger.info(f"Heuristically extracted {len(items)} items from preamble text")

                elif norm_name in ["task", "webfetch", "bash"]:
                    # For simple string tools, use the content after the function tag
                    # (This is a common failure mode where model puts everything after the tag)
                    remaining_text = func_content.strip()
                    if remaining_text:
                        param_map = {
                            "task": "prompt",
                            "webfetch": "url",
                            "bash": "command"
                        }
                        arguments[param_map[func_name]] = remaining_text
                        logger.info(f"Heuristically assigned remaining text to {param_map[func_name]}")

            if func_name:
                call = {
                    "id": f"call_{uuid.uuid4().hex[:8]}",
                    "type": "function",
                    "function": {
                        "name": func_name,
                        "arguments": json.dumps(arguments)
                    }
                }
                tool_calls.append(call)
                logger.info(f"Successfully parsed tool call: {func_name} with {len(arguments)} args")

        return len(tool_calls) > 0, tool_calls

    def _parse_tool_call_block(self, content: str) -> Optional[Dict[str, Any]]:
        """Parse a single <tool_call>...</tool_call> block"""
        # Extract function name and parameters
        func_match = self.function_regex.search(content)
        if not func_match:
            return None

        # Get function name (from group 1 or 3)
        func_name = func_match.group(1) or func_match.group(3)
        if not func_name:
            return None

        func_name = func_name.strip()

        # Get the content between <function=name> and </function>
        func_content = func_match.group(2) or func_match.group(4)

        # Parse parameters
        arguments = {}
        if func_content:
            param_matches = self.parameter_regex.finditer(func_content)
            for param_match in param_matches:
                param_name = param_match.group(1)
                param_value = param_match.group(2)

                if param_name:
                    param_name = param_name.strip()
                    if param_value:
                        param_value = param_value.strip()
                        # Try to parse as JSON if it looks like a list or dict
                        if (param_value.startswith("[") and param_value.endswith("]")) or \
                           (param_value.startswith("{") and param_value.endswith("}")):
                            try:
                                # Clean up common model artifacts (like escaped quotes or backticks)
                                cleaned_value = param_value.strip("`").strip()
                                # Verify it is valid JSON by parsing it
                                parsed_obj = json.loads(cleaned_value)
                                
                                # OpenAI format REQUIRES that 'arguments' is a JSON STRING.
                                # Most clients expect that if a parameter is an object/array,
                                # it is already encoded as part of that string.
                                # If we keep it as a dict/list here, json.dumps(arguments) 
                                # below will turn it into a JSON array/object inside the string.
                                # Example: {"todos": [{"task": "..."}]}
                                # This is usually what is expected. 
                                # The previous error "expected array, received string" is confusing
                                # if it came from a Zod-like validator on the client side.
                                
                                # Let's keep it as an object and see if the double-encoding was the issue.
                                arguments[param_name] = parsed_obj
                            except json.JSONDecodeError:
                                # Try literal_eval for single quotes or other Python-isms
                                try:
                                    arguments[param_name] = ast.literal_eval(param_value)
                                except:
                                    arguments[param_name] = param_value
                        else:
                            arguments[param_name] = param_value

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
