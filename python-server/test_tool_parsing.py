import json
from tool_parser import parse_tool_calls

def test_todowrite_parsing():
    output = """I'll create a todo list to track this implementation.
<tool_call>
<function=todowrite>
<parameter=todos>
[{"task": "Create dark mode toggle component", "status": "pending"}, {"task": "Add dark mode state management", "status": "pending"}]
</parameter>
</function>
</tool_call>"""
    
    has_calls, tool_calls = parse_tool_calls(output)
    
    print(f"Has calls: {has_calls}")
    print(f"Tool calls: {json.dumps(tool_calls, indent=2)}")
    
    if has_calls:
        args = json.loads(tool_calls[0]["function"]["arguments"])
        if isinstance(args["todos"], list):
            print("SUCCESS: 'todos' is a list")
        else:
            print(f"FAILURE: 'todos' is {type(args['todos'])}")
            print(f"Value: {args['todos']}")

if __name__ == "__main__":
    test_todowrite_parsing()
