#!/usr/bin/env python3


def snapshot(file_path):
    with open(file_path, "r") as f:
        lines = f.readlines()

    # Get the third line (index 2)
    if len(lines) < 3:
        raise ValueError(f"File has less than 3 lines")

    third_line = lines[2]

    # Check if it starts with 'expression: '
    prefix = "expression: "
    if not third_line.startswith(prefix):
        raise ValueError(f"Third line does not start with '{prefix}'")

    # Remove the prefix and any trailing newline
    expression_value = third_line[len(prefix) :].rstrip("\n")

    # The value is a JSON-encoded string, so we need to decode it
    import json

    deserialized_value = json.loads(expression_value)

    print(deserialized_value)
