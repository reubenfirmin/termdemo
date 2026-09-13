"""Small fail-closed source inventory for this dependency-free Rust repository.

This is a lexical extraction guard, not a Rust compiler or a motion approval.
It compares complete existing declarations, ignoring comments/whitespace and
the pub(super) visibility needed to move a declaration across a module boundary.
The compiler, dense camera traces and normal-render comparisons are separate gates.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re

LEXEME = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|b?"(?:\\.|[^"\\])*"|'
                    r"b?'(?:\\.|[^'\\])'|[A-Za-z_][A-Za-z_0-9]*|[0-9][A-Za-z_0-9]*|[^\s]")
DECLARATION = re.compile(
    r'(?m)^(?:pub(?:\([^)]*\))? )?'
    r'(?:(?:const |unsafe |extern "[^"\n]+" )*fn (?P<fn>\w+)'
    r'|const (?P<const>\w+)|static (?:mut )?(?P<static>\w+)'
    r'|struct (?P<struct>\w+)|enum (?P<enum>\w+)|impl (?P<impl>\w+))')


def lexemes(text):
    return [match for match in LEXEME.finditer(text)
            if not match.group().startswith(("//", "/*"))]


def declarations(source):
    items = []
    for match in DECLARATION.finditer(source):
        name = next(value for value in match.groupdict().values() if value is not None)
        kind = next(key for key, value in match.groupdict().items() if value is not None)
        start = match.start()
        # Attach the declaration's attributes, not attributes from another item.
        while start:
            previous = source.rfind("\n", 0, start - 1) + 1
            if source[previous:start].strip().startswith("#["):
                start = previous
            else:
                break
        depth = 0
        brackets = 0
        parentheses = 0
        opened = False
        end = None
        for token in lexemes(source[match.end():]):
            value = token.group()
            if value == "[":
                brackets += 1
            elif value == "]":
                brackets -= 1
            elif value == "(":
                parentheses += 1
            elif value == ")":
                parentheses -= 1
            elif value == "{":
                depth += 1
                opened = True
            elif value == "}":
                depth -= 1
                if depth < 0:
                    raise ValueError(f"unbalanced declaration {name}")
                if depth == 0 and opened and kind in ("fn", "struct", "enum", "impl"):
                    end = match.end() + token.end()
                    break
            elif value == ";" and depth == 0 and brackets == 0 and parentheses == 0 and kind in ("const", "static"):
                end = match.end() + token.end()
                break
        if end is None:
            raise ValueError(f"could not delimit declaration {name}")
        # Keep inter-item whitespace/comments for reversible mechanical moves.
        while end < len(source) and source[end] in " \t\r\n":
            end += 1
        items.append(dict(name=name, kind=kind, start=start, end=end,
                          text=source[start:end], audit='#[cfg(feature = "phase0-audit")]'
                          in source[start:match.start()]))
    for first, second in zip(items, items[1:]):
        if first["end"] > second["start"]:
            raise ValueError(f"overlapping declarations {first['name']}/{second['name']}")
    return items


def declaration_digest(text):
    text = re.sub(r"(?m)^([ \t]*)pub\(super\) ", r"\1", text)
    tokens = [token.group() for token in lexemes(text)]
    return hashlib.sha256(json.dumps(tokens, separators=(",", ":")).encode()).hexdigest()


def inventory(paths):
    result = {}
    for path in paths:
        for item in declarations(path.read_text()):
            key = item["kind"] + ":" + item["name"]
            if key in result:
                raise ValueError(f"duplicate top-level declaration {key}")
            result[key] = dict(sha256=declaration_digest(item["text"]), source=str(path))
    return result


def check_extraction(root: Path):
    reference = json.loads((root / "tests/phase1_extraction.json").read_text())
    # The withdrawn standalone speed experiment has unrelated test/module items.
    paths = sorted(path for path in (root / "src").glob("*.rs") if path.name != "flight_speed.rs")
    actual = inventory(paths)
    failures = [key for key, expected in reference["declarations"].items()
                if key not in actual or actual[key]["sha256"] != expected]
    if failures:
        raise RuntimeError("extraction changed existing declaration tokens: " + ", ".join(failures))
    documented = int("audit_only_entrypoint_change" in reference)
    return (f"{len(reference['declarations']) - documented} original declarations unchanged "
            f"(visibility/comments/whitespace excluded); {documented} documented audit-only "
            "entrypoint update; not canonical-flight approval")


class PipelineSource:
    """Read actual module declarations; legacy locks do not depend on file order."""
    def __init__(self, paths):
        sources = [re.sub(r"(?m)^([ \t]*)pub\(super\) ", r"\1", path.read_text())
                   for path in paths]
        self.text = "\n".join(sources)
        self.items = {}
        for source in sources:
            for item in declarations(source):
                key = (item["kind"], item["name"])
                if key in self.items:
                    raise ValueError(f"duplicate pipeline item {key}")
                self.items[key] = item["text"]

    def function(self, name):
        text = self.items[("fn", name)]
        return text[text.index("fn " + name + "("):].rstrip()

    def block(self, *names, trailing="\n"):
        # Preserve the legacy span's inter-function layout while allowing those
        # unchanged definitions to live in separate real modules. Existing
        # expected SHA256 values are NOT replaced by extraction hashes.
        return "\n\n".join(self.function(name) for name in names) + trailing


if __name__ == "__main__":
    print(check_extraction(Path(__file__).resolve().parent.parent))
