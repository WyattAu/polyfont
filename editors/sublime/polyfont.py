import json
import os
import sublime
import sublime_plugin

try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib
    except ModuleNotFoundError:
        tomllib = None

CONFIG_FILENAME = ".polyfont.toml"
THEME_NAME = "Polyfont.sublime-theme"

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


def _find_config(window):
    folders = window.folders()
    for folder in folders:
        candidate = os.path.join(folder, CONFIG_FILENAME)
        if os.path.isfile(candidate):
            return candidate
    home = os.path.expanduser("~")
    home_cfg = os.path.join(home, CONFIG_FILENAME)
    if os.path.isfile(home_cfg):
        return home_cfg
    return None


def _build_font_face(family, weight=None, style=None):
    face = family
    if weight and weight != "regular" and weight in WEIGHT_SUFFIX_MAP:
        face += WEIGHT_SUFFIX_MAP[weight]
    if style == "italic":
        face += " Italic"
    elif style == "oblique":
        face += " Oblique"
    return face


def _parse_config(config_path):
    if tomllib is None:
        return None
    with open(config_path, "rb") as f:
        return tomllib.load(f)


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
                "class": "label",
                "scope": scope,
                "settings": {"font.face": font_face},
            }
        )

    if fallbacks:
        entries.append(
            {
                "class": "label",
                "settings": {
                    "font.face": ", ".join(
                        [default_face] + fallbacks
                    )
                },
            }
        )

    return entries


class PolyfontApplyCommand(sublime_plugin.TextCommand):
    def run(self, edit):
        if tomllib is None:
            sublime.error_message(
                "Polyfont: Python 3.11+ required (tomllib not available).\n"
                "Install the 'tomli' package or use Sublime Text 4 with Python 3.11+."
            )
            return

        window = self.view.window()
        if not window:
            return

        config_path = _find_config(window)
        if not config_path:
            sublime.error_message(
                f"Polyfont: No {CONFIG_FILENAME} found in project folders or home directory."
            )
            return

        try:
            config = _parse_config(config_path)
        except Exception as e:
            sublime.error_message(f"Polyfont: Failed to parse config: {e}")
            return

        if not config:
            sublime.error_message("Polyfont: Failed to parse config.")
            return

        entries = _build_theme_entries(config)
        if not entries:
            sublime.status_message("Polyfont: No rules found in config.")
            return

        theme_json = json.dumps(entries, indent=4)

        packages_path = sublime.packages_path()
        user_path = os.path.join(packages_path, "User")
        os.makedirs(user_path, exist_ok=True)
        theme_path = os.path.join(user_path, THEME_NAME)

        with open(theme_path, "w", encoding="utf-8") as f:
            f.write(theme_json)

        settings = sublime.load_settings("Preferences.sublime-settings")
        settings.set("theme", THEME_NAME)
        sublime.save_settings("Preferences.sublime-settings")

        sublime.status_message(
            f"Polyfont: Applied {len(entries)} font rules from {os.path.basename(config_path)}"
        )


class PolyfontGenerateCommand(sublime_plugin.TextCommand):
    def run(self, edit):
        window = self.view.window()
        if not window:
            return

        folders = window.folders()
        if not folders:
            sublime.error_message("Polyfont: Open a project folder first.")
            return

        target = os.path.join(folders[0], CONFIG_FILENAME)
        if os.path.isfile(target):
            if not sublime.ok_cancel_dialog(
                f"{CONFIG_FILENAME} already exists. Overwrite?"
            ):
                return

        sample = """\
# Polyfont Configuration
# See: https://github.com/WyattAu/polyfont

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

[[rules]]
scope = "entity.name.function"
[rules.font]
family = "Monaspace Argon"
weight = "semi-bold"

[[rules]]
scope = "string"
[rules.font]
family = "Source Code Pro"

[[rules]]
scope = "variable"
[rules.font]
family = "JetBrains Mono"
"""
        with open(target, "w", encoding="utf-8") as f:
            f.write(sample)

        new_view = window.open_file(target)
        sublime.status_message(f"Polyfont: Generated {CONFIG_FILENAME}")
