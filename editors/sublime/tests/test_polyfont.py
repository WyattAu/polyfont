import unittest
import tempfile
import os
import sys
import json

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))


def _build_font_face(family, weight=None, style=None):
    WEIGHT_SUFFIX_MAP = {
        "thin": " Thin",
        "extra-light": " ExtraLight",
        "light": " Light",
        "medium": " Medium",
        "semi-bold": " SemiBold",
        "bold": " Bold",
        "extra-bold": " ExtraBold",
        "black": " Black",
    }

    face = family
    if weight and weight != "regular" and weight in WEIGHT_SUFFIX_MAP:
        face += WEIGHT_SUFFIX_MAP[weight]
    if style == "italic":
        face += " Italic"
    elif style == "oblique":
        face += " Oblique"
    return face


def _build_theme_entries(config):
    entries = []
    default_family = config.get("default", {}).get("family", "monospace")
    fallbacks = config.get("default", {}).get("fallbacks", [])
    default_face = _build_font_face(
        default_family,
        config.get("default", {}).get("weight"),
        config.get("default", {}).get("style"),
    )

    for rule in config.get("rules", []):
        scope = rule.get("scope", "")
        font = rule.get("font", {})
        family = font.get("family", default_family)
        weight = font.get("weight")
        style = font.get("style")
        font_face = _build_font_face(family, weight, style)
        entries.append(
            {
                "class": "invisible",
                "scope": scope,
                "settings": {"font.face": font_face},
            }
        )

    if fallbacks:
        entries.append(
            {
                "class": "invisible",
                "settings": {
                    "font.face": ", ".join(
                        [default_face] + fallbacks
                    )
                },
            }
        )

    return entries


class TestBuildFontFace(unittest.TestCase):
    def test_plain_family(self):
        self.assertEqual(_build_font_face("Fira Code"), "Fira Code")

    def test_bold_weight(self):
        self.assertEqual(_build_font_face("Fira Code", weight="bold"), "Fira Code Bold")

    def test_semi_bold_weight(self):
        self.assertEqual(_build_font_face("Fira Code", weight="semi-bold"), "Fira Code SemiBold")

    def test_extra_bold_weight(self):
        self.assertEqual(_build_font_face("Fira Code", weight="extra-bold"), "Fira Code ExtraBold")

    def test_black_weight(self):
        self.assertEqual(_build_font_face("Fira Code", weight="black"), "Fira Code Black")

    def test_light_weight(self):
        self.assertEqual(_build_font_face("Fira Code", weight="light"), "Fira Code Light")

    def test_thin_weight(self):
        self.assertEqual(_build_font_face("Fira Code", weight="thin"), "Fira Code Thin")

    def test_extra_light_weight(self):
        self.assertEqual(_build_font_face("Fira Code", weight="extra-light"), "Fira Code ExtraLight")

    def test_medium_weight(self):
        self.assertEqual(_build_font_face("Fira Code", weight="medium"), "Fira Code Medium")

    def test_regular_weight_omitted(self):
        self.assertEqual(_build_font_face("Fira Code", weight="regular"), "Fira Code")

    def test_italic_style(self):
        self.assertEqual(_build_font_face("Fira Code", style="italic"), "Fira Code Italic")

    def test_oblique_style(self):
        self.assertEqual(_build_font_face("Fira Code", style="oblique"), "Fira Code Oblique")

    def test_bold_italic(self):
        self.assertEqual(
            _build_font_face("Fira Code", weight="bold", style="italic"),
            "Fira Code Bold Italic",
        )

    def test_normal_style_omitted(self):
        self.assertEqual(_build_font_face("Fira Code", style="normal"), "Fira Code")

    def test_unknown_weight_ignored(self):
        self.assertEqual(_build_font_face("Fira Code", weight="ultra-bold"), "Fira Code")

    def test_none_weight_same_as_plain(self):
        self.assertEqual(_build_font_face("Fira Code", weight=None), "Fira Code")

    def test_none_style_same_as_plain(self):
        self.assertEqual(_build_font_face("Fira Code", style=None), "Fira Code")


class TestBuildThemeEntries(unittest.TestCase):
    def test_empty_config(self):
        config = {}
        entries = _build_theme_entries(config)
        self.assertEqual(len(entries), 0)

    def test_single_rule(self):
        config = {
            "rules": [
                {"scope": "keyword", "font": {"family": "Maple Mono", "weight": "bold"}}
            ]
        }
        entries = _build_theme_entries(config)
        self.assertEqual(len(entries), 1)
        self.assertEqual(entries[0]["scope"], "keyword")
        self.assertEqual(entries[0]["settings"]["font.face"], "Maple Mono Bold")
        self.assertEqual(entries[0]["class"], "invisible")

    def test_multiple_rules(self):
        config = {
            "rules": [
                {"scope": "keyword", "font": {"family": "A"}},
                {"scope": "comment", "font": {"family": "B", "style": "italic"}},
            ]
        }
        entries = _build_theme_entries(config)
        self.assertEqual(len(entries), 2)
        self.assertEqual(entries[0]["scope"], "keyword")
        self.assertEqual(entries[1]["scope"], "comment")
        self.assertEqual(entries[1]["settings"]["font.face"], "B Italic")

    def test_default_family_used_when_rule_missing_family(self):
        config = {"default": {"family": "Fira Code"}, "rules": [{"scope": "keyword", "font": {}}]}
        entries = _build_theme_entries(config)
        self.assertEqual(entries[0]["settings"]["font.face"], "Fira Code")

    def test_fallback_entry_added(self):
        config = {"default": {"family": "Fira Code", "fallbacks": ["JetBrains Mono", "monospace"]}}
        entries = _build_theme_entries(config)
        fallback_entry = entries[-1]
        self.assertNotIn("scope", fallback_entry)
        self.assertEqual(fallback_entry["settings"]["font.face"], "Fira Code, JetBrains Mono, monospace")

    def test_fallback_uses_default_weight_and_style(self):
        config = {
            "default": {"family": "Fira Code", "weight": "bold", "style": "italic", "fallbacks": ["monospace"]},
        }
        entries = _build_theme_entries(config)
        fallback_entry = entries[-1]
        self.assertEqual(fallback_entry["settings"]["font.face"], "Fira Code Bold Italic, monospace")

    def test_no_fallback_when_empty(self):
        config = {"default": {"family": "Fira Code"}, "fallbacks": []}
        entries = _build_theme_entries(config)
        for entry in entries:
            self.assertIn("scope", entry)

    def test_rule_inherits_default_with_weight(self):
        config = {
            "default": {"family": "Fira Code"},
            "rules": [
                {"scope": "keyword", "font": {"family": "Maple Mono", "weight": "semi-bold"}}
            ],
        }
        entries = _build_theme_entries(config)
        self.assertEqual(entries[0]["settings"]["font.face"], "Maple Mono SemiBold")

    def test_entries_are_valid_json(self):
        config = {
            "rules": [
                {"scope": "keyword", "font": {"family": "A"}},
                {"scope": "comment", "font": {"family": "B"}},
            ]
        }
        entries = _build_theme_entries(config)
        json_str = json.dumps(entries)
        parsed = json.loads(json_str)
        self.assertEqual(len(parsed), 2)

    def test_multiple_rules_same_scope(self):
        config = {
            "rules": [
                {"scope": "string", "font": {"family": "A"}},
                {"scope": "string", "font": {"family": "B"}},
            ]
        }
        entries = _build_theme_entries(config)
        self.assertEqual(len(entries), 2)
        self.assertEqual(entries[0]["settings"]["font.face"], "A")
        self.assertEqual(entries[1]["settings"]["font.face"], "B")


class TestConfigParsingRoundTrip(unittest.TestCase):
    def test_parse_and_build_toml_config(self):
        toml_content = b"""
version = 1

[default]
family = "Fira Code"
fallbacks = ["JetBrains Mono", "monospace"]

[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
weight = "bold"

[[rules]]
scope = "comment"
[rules.font]
family = "IBM Plex Mono"
style = "italic"
"""
        try:
            import tomllib
        except ModuleNotFoundError:
            import tomli as tomllib

        fd, tmppath = tempfile.mkstemp(suffix=".toml")
        try:
            with open(fd, "wb") as f:
                f.write(toml_content)
            with open(tmppath, "rb") as f:
                config = tomllib.load(f)

            self.assertEqual(config["version"], 1)
            self.assertEqual(config["default"]["family"], "Fira Code")
            self.assertEqual(len(config["rules"]), 2)
            self.assertEqual(config["rules"][0]["scope"], "keyword")
            self.assertEqual(config["rules"][0]["font"]["family"], "Maple Mono")
            self.assertEqual(config["rules"][0]["font"]["weight"], "bold")
            self.assertEqual(config["rules"][1]["font"]["style"], "italic")

            entries = _build_theme_entries(config)
            self.assertEqual(len(entries), 3)
            self.assertEqual(entries[0]["settings"]["font.face"], "Maple Mono Bold")
        finally:
            os.unlink(tmppath)

    def test_empty_toml_file(self):
        try:
            import tomllib
        except ModuleNotFoundError:
            import tomli as tomllib

        fd, tmppath = tempfile.mkstemp(suffix=".toml")
        try:
            with open(fd, "wb") as f:
                f.write(b"")
            with open(tmppath, "rb") as f:
                config = tomllib.load(f)
        except Exception:
            config = {}

        self.assertEqual(len(_build_theme_entries(config)), 0)
        os.unlink(tmppath)


if __name__ == "__main__":
    unittest.main()
