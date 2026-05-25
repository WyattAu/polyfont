# RFC: Per-Highlight-Group Font Family Support

- **Author**: Wyatt Au
- **Date**: 2026-05-25
- **Status**: Draft
- **Target**: neovim/neovim

## Summary

Add an optional `font` field to highlight group definitions set via
`nvim_set_hl()`, allowing GUI clients to render different font families per
highlight group. This is a single, additive API change that enables rich
typographic control -- such as per-token font rendering -- for all GUI clients
without requiring plugin-specific workarounds.

## Motivation

### Current state

Neovim's highlight system already supports per-highlight-group text
attributes:

- `bold`
- `italic`
- `underline`
- `strikethrough`
- `undercurl`
- `underdouble`
- `underdotted`
- `underdashed`
- `foreground` / `background` colors

Notably absent is **font family**. There is no mechanism to tell a GUI client
to render one highlight group in "Maple Mono" and another in "IBM Plex Mono".
This is a surprising gap, given that font family is one of the most fundamental
typographic properties and all major GUI toolkits support it natively.

### Use case: per-token font highlighting (polyfont)

Per-token font highlighting -- sometimes called "polyfont" -- assigns different
font families to different syntactic categories:

- Keywords in **Maple Mono Bold**
- Comments in *IBM Plex Mono Italic*
- Functions in **Monaspace Argon**
- Strings in Source Code Pro Light
- Constants in Monaspace Radon

This goes beyond color-based highlighting. Font family variation provides an
additional visual channel that improves code readability, particularly for
developers with color vision deficiency or those working in contexts where
color alone is insufficient to distinguish token types.

VSCode has supported this natively since version 1.97 (December 2025) via the
`fontFamily` field in `textMateRules`. Neovim has no equivalent.

### Current workaround

The polyfont project currently works around this limitation in Neovim by:

1. Creating highlight groups with only `bold` and `italic` attributes via
   `nvim_set_hl()`.
2. Storing font metadata (family, weight, style, fallbacks) in a global
   variable `vim.g.polyfont_font_map`.
3. Applying highlights to tokens via Tree-sitter captures and extmarks using
   `nvim_set_decoration_provider`.
4. Requiring each GUI client to read `vim.g.polyfont_font_map` and implement
   its own font-switching logic based on extmark highlight group names.

This approach has several drawbacks:

- **GUI-specific implementation required**: Each GUI (Goneovim, Neovide, etc.)
  must independently implement font-map reading and per-character font
  switching. There is no shared contract.
- **No standard protocol**: Font metadata is communicated via an ad-hoc global
  variable rather than through the established `hl_attr_define` UI event.
- **Redundant round trips**: The GUI must correlate highlight group IDs back to
  font metadata rather than receiving font information directly in the
  highlight attribute definition.
- **Fragile coupling**: The plugin must manage extmark lifecycle, decoration
  providers, and font metadata synchronization manually.

### Benefit

A single `font` field on highlight definitions would:

- Establish a standard protocol for per-group font specification.
- Eliminate per-GUI workaround implementations.
- Enable any plugin to leverage typographic variation, not just polyfont.
- Align Neovim's highlight expressiveness with VSCode's `textMateRules`.
- Remain fully backward compatible -- GUIs that ignore the field are unaffected.

## Specification

### API

The `nvim_set_hl()` function accepts a new optional `font` key in the
highlight definition dictionary:

```lua
-- Single font family
vim.api.nvim_set_hl(0, "KeywordFont", {
  font = "Maple Mono",
  bold = true,
})

-- Font stack with fallbacks (array of strings)
vim.api.nvim_set_hl(0, "CommentFont", {
  font = { "IBM Plex Mono", "JetBrains Mono", "monospace" },
  italic = true,
})

-- Unset / inherit from guifont (default behavior)
vim.api.nvim_set_hl(0, "Normal", {
  font = nil,  -- explicit nil, same as omitting the key
})
```

### Field semantics

| Value | Behavior |
|---|---|
| `nil` (default) | Inherit font from global `guifont` / `guifontwide` |
| `"Family Name"` (string) | Use the specified font family |
| `{ "Family A", "Family B" }` (array) | Use the first available family as a fallback stack |

### Constraints

- The `font` field specifies font **family** only. Size, weight, and style are
  controlled by existing fields (`bold`, `italic`) or global settings
  (`guifont`). See Open Questions for discussion of extended specs.
- When `font` is set to a family not available on the system, the GUI falls
  back to the global `guifont` family, consistent with how missing fonts are
  handled elsewhere.
- The `font` field is advisory. GUI clients that do not support per-highlight
  font rendering silently ignore it.

### Highlight group query

`nvim_get_hl()` returns the `font` field when present:

```lua
local hl = vim.api.nvim_get_hl(0, { name = "KeywordFont" })
-- hl.font == "Maple Mono" or { "IBM Plex Mono", ... }
```

### UI event protocol

The `hl_attr_define` UI event gains a `font` field:

```
hl_attr_define {id, rgb_attrs, cterm_attrs, info}
```

The `rgb_attrs` (and/or `info`) dictionary now includes:

```json
{
  "foreground": 16777215,
  "bold": true,
  "font": "Maple Mono"
}
```

Or for a font stack:

```json
{
  "font": ["IBM Plex Mono", "JetBrains Mono", "monospace"]
}
```

GUI clients that do not recognize the `font` key ignore it without error,
preserving backward compatibility at the protocol level.

### Behavior with extmarks

When an extmark applies a highlight group that carries a `font` field, the GUI
renders the marked region using that font family. This integrates naturally
with the existing extmark + highlight group pipeline. No changes to the
extmark API are required.

### Default value and inheritance

The default value of `font` for all highlight groups is `nil`, meaning
"inherit from the global font." This matches the behavior of all existing
highlight attributes and ensures that no existing configuration is affected.

Highlight groups linked via `link =` inherit the linked group's `font` field,
consistent with how `bold`, `italic`, and color attributes propagate through
links. If a group sets `font` explicitly, the explicit value takes precedence
over the linked group's font.

## Implementation Sketch

### 1. HlAttrs struct

**File**: `src/nvim/highlight_group.c`

Add a `font_name` field to the `HlAttrs` struct:

```c
typedef struct {
  // ... existing fields ...
  bool bold;
  bool italic;
  // ... more existing fields ...

  // New field
  char *font_name;       // single family, or NULL for inherit
  char **font_stack;     // NULL-terminated array for fallback stack
  size_t font_stack_len; // number of entries in font_stack
} HlAttrs;
```

`font_name` and `font_stack` are mutually exclusive in practice:
- If the user passes a string, `font_name` is set and `font_stack` is NULL.
- If the user passes an array, `font_stack` is populated and `font_name` is
  NULL.
- If the user passes nil or omits the key, both are NULL (inherit).

### 2. API parsing

**File**: `src/nvim/api/vim.c`

In the `nvim_set_hl` implementation, parse the `font` key from the highlight
definition dictionary:

```c
// Pseudocode
if (dict_has_key(dict, "font")) {
    Object font_obj = dict_get(dict, "font");
    if (font_obj.type == kObjectTypeString) {
        attrs.font_name = xstrdup(font_obj.data.string.data);
    } else if (font_obj.type == kObjectTypeArray) {
        // Parse array of strings into font_stack
        Array arr = font_obj.data.array;
        attrs.font_stack = xmalloc(sizeof(char*) * (arr.size + 1));
        for (size_t i = 0; i < arr.size; i++) {
            attrs.font_stack[i] = xstrdup(arr.items[i].data.string.data);
        }
        attrs.font_stack[arr.size] = NULL;
        attrs.font_stack_len = arr.size;
    }
}
```

### 3. Grid / UI event propagation

**File**: `src/nvim/grid.c`

When constructing the `hl_attr_define` UI event, include the `font` field in
the attributes dictionary sent to UI clients:

```c
// In the function that builds the hl_attr_define event args
if (attrs.font_name) {
    PUT(attrs_dict, "font", STRING_OBJ(cstr_as_string(attrs.font_name)));
} else if (attrs.font_stack) {
    Array font_arr = ARRAY_DICT_INIT;
    for (size_t i = 0; i < attrs.font_stack_len; i++) {
        ADD(font_arr, STRING_OBJ(cstr_as_string(attrs.font_stack[i])));
    }
    PUT(attrs_dict, "font", ARRAY_OBJ(font_arr));
}
```

### 4. Memory management

Font strings are allocated via `xstrdup` and freed when highlight groups are
redefined or cleared. This follows the existing pattern for highlight attribute
lifecycle management.

### 5. Highlight group retrieval

**File**: `src/nvim/api/vim.c`

In `nvim_get_hl`, serialize the `font` field back into the return dictionary.
Return a string for single-family fonts, an array for font stacks, and omit
the key entirely when `font` is unset (nil).

### Files touched

| File | Change |
|---|---|
| `src/nvim/highlight_group.c` | Add `font_name` / `font_stack` to `HlAttrs` |
| `src/nvim/highlight_group.h` | Update `HlAttrs` struct definition |
| `src/nvim/api/vim.c` | Parse `font` key in `nvim_set_hl`; serialize in `nvim_get_hl` |
| `src/nvim/grid.c` | Include `font` in `hl_attr_define` UI event |
| `src/nvim/grid.h` | Update grid highlight structs if needed |
| `test/functional/api/highlight_spec.lua` | New test cases |
| `runtime/lua/vim/highlight.lua` | Documentation updates |

## Compatibility

### Backward compatibility

This change is fully backward compatible:

- **API level**: The `font` key is optional. Existing calls to `nvim_set_hl`
  that do not include `font` behave identically. The `nvim_get_hl` return
  value gains a new key only when `font` was explicitly set, which is
  non-breaking for consumers that ignore unknown keys.
- **Protocol level**: The `hl_attr_define` UI event adds a `font` field to the
  attributes dictionary. GUI clients written before this change do not read
  this field and are unaffected. The msgpack-rpc specification allows
  additional keys in dictionaries without breaking existing deserialization.
- **Behavioral level**: All highlight groups default to `font = nil`, meaning
  "inherit from guifont." No visual change occurs unless a user or plugin
  explicitly sets `font` on a highlight group.
- **TUI**: The built-in TUI does not support per-character font switching and
  ignores the `font` field. This is expected and acceptable. Font family
  variation is inherently a GUI capability.

### GUI adoption

GUI clients can adopt this feature incrementally:

1. **No action required**: GUIs that ignore the `font` field continue to work
   correctly with global `guifont`.
2. **Basic adoption**: GUIs read the `font` field from `hl_attr_define` and
   switch font family when rendering highlighted regions.
3. **Full adoption**: GUIs implement font stack resolution (trying each family
   in order until one is available).

### Plugin compatibility

Existing plugins that call `nvim_set_hl` are unaffected. Plugins that read
highlight definitions via `nvim_get_hl` will see a new `font` key in some
groups. Well-written plugins already ignore unknown keys in highlight
definitions, so this should not cause issues.

## Alternatives Considered

### 1. Status quo: per-GUI workaround implementations

**Description**: Continue using the current polyfont approach -- store font
metadata in `vim.g` and require each GUI to implement its own font-switching
logic.

**Drawbacks**:

- Fragmented implementation across GUI clients.
- No standard protocol; each GUI interprets the ad-hoc metadata differently.
- Plugin complexity: must manage both highlight groups and out-of-band font
  metadata.
- No discoverability: GUIs must know to look for `vim.g.polyfont_font_map`
  specifically.

**Conclusion**: Acceptable as a short-term measure, but does not scale.

### 2. Font substitution via Unicode ranges (Kitty approach)

**Description**: Kitty's `symbol_map` feature maps Unicode ranges to font
families. This is configured at the terminal level, not the editor level.

**Drawbacks**:

- Scope-based, not token-based. You cannot map "all keywords" to a font; you
  can only map "Unicode codepoints U+XXXX-U+YYYY" to a font.
- Requires mapping tokens to Private Use Area (PUA) codepoints, which is
  complex and fragile.
- Cannot handle overlapping scopes (e.g., a keyword inside a string).
- Not applicable to GUI clients outside Kitty.

**Conclusion**: Not suitable for per-highlight-group font assignment.

### 3. Custom renderer plugin via remote UI

**Description**: Implement a custom remote UI (msgpack-rpc client) that
  intercepts rendering and applies font switching client-side.

**Drawbacks**:

- Heavyweight: requires a full remote UI implementation.
- Fragile: must stay in sync with Neovim's UI protocol changes.
- High barrier to entry: not accessible to most plugin authors.
- Does not benefit other GUI clients.

**Conclusion**: Overengineered for the problem at hand.

### 4. Extended font specification in highlight groups

**Description**: Instead of just `font` (family), accept a full font
specification including size, weight, and style:

```lua
vim.api.nvim_set_hl(0, "KeywordFont", {
  font = { family = "Maple Mono", weight = "bold", size = 14 },
})
```

**Drawbacks**:

- Overlaps with existing `bold` and `italic` fields, creating ambiguity.
- Increases implementation complexity for GUI clients.
- Size variation per highlight group may cause alignment issues in a grid-based
  renderer.

**Conclusion**: Too broad for an initial implementation. Family-only is the
minimal useful change. Extended specs can be considered in a follow-up RFC.

## Open Questions

1. **Font stack or single family only?**
   Should `font` accept an array of strings (fallback stack) in the initial
   implementation, or should it be restricted to a single string to reduce
   complexity? A font stack is more expressive but adds implementation burden
   for GUI clients. A reasonable compromise: accept only strings initially, add
   array support in a follow-up.

2. **Extended font specification?**
   Should `font` eventually accept a full specification (family + size + weight
   + style) as a dictionary, or remain family-only? If extended, how should
   conflicts with `bold` and `italic` be resolved? This RFC proposes family-only
   as the initial scope.

3. **Rendering performance impact?**
   Frequent font switching within a single line may impact rendering performance
   in GUI clients, particularly those using GPU-accelerated text rendering.
   Should the implementation impose a practical limit on the number of distinct
   font families used simultaneously, or leave performance optimization to GUI
   clients?

4. **Interaction with `guifont` and `guifontwide`?**
   When `font` is set on a highlight group, does it replace both `guifont` and
   `guifontwide` for that group, or only `guifont`? How should wide characters
   be handled when a per-group font is active but the specified family lacks
   CJK glyphs?

5. **TUI considerations?**
   The built-in TUI cannot render different fonts per highlight group in a
   standard terminal. Should `nvim_set_hl` silently accept `font` in TUI mode,
   or emit a warning? This RFC proposes silent acceptance (consistent with how
   GUI-only attributes like `undercurl` are handled).

6. **Namespace for highlight groups?**
   Should plugins be encouraged to use a naming convention (e.g.,
   `@polyfont.keyword`) for highlight groups that carry font information, or
   should any highlight group be allowed to set `font`? This RFC proposes no
   restriction -- any highlight group may set `font`.
