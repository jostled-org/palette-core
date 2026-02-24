# CSS variable naming overhaul

## Problem

Current CSS variables are verbose and inconsistent. The naming follows a mechanical `--{prefix}-{section}-{field}` pattern that mirrors the Rust struct hierarchy, producing names optimized for code navigation rather than CSS authoring. The prefix is required.

```css
/* 22–41 characters per variable */
body {
    background: var(--app-base-background);
    color: var(--app-base-foreground);
    border-color: var(--app-base-border-highlight);
}

.error { color: var(--app-semantic-error); }
.keyword { color: var(--app-syntax-keywords); }
.diagnostic { border-bottom-color: var(--app-editor-diagnostic-underline-error); }
```

Issues:

- Prefix is mandatory, adding noise when only one theme system exists on the page
- `base-` prefix on the most-used variables adds further noise
- `semantic-` prefix on `error`, `success`, etc. is redundant — these names are unambiguous
- Section names vary in length: `typography` (10), `diff` (4)
- Slot names are inconsistent: `keywords` (plural) vs `comment` (singular)
- Worst-case variable is 41 characters (`--app-editor-diagnostic-underline-error`)

## Proposed naming

Prefix becomes optional (default: none). Core colors (base + semantic) drop the section prefix. All other sections use short 2–4 character prefixes. Slot names use consistent abbreviations and singular nouns.

Pattern: `--{slot}` for core, `--{section}-{slot}` for grouped. With optional prefix: `--{prefix}-{slot}` and `--{prefix}-{section}-{slot}`.

### API change

```rust
// No prefix (default) — cleanest output
let css = palette.to_css(None);
// --bg: #1A1B26;
// --syn-keyword: #BB9AF7;

// With prefix — for namespace safety
let css = palette.to_css(Some("app"));
// --app-bg: #1A1B26;
// --app-syn-keyword: #BB9AF7;
```

### When to use a prefix

A prefix prevents collisions when another CSS variable system is on the same page. Without one, bare names like `--bg` or `--error` could clash.

```css
/* Two palette-core instances on the same page (e.g. split-pane editor) */
.pane-left  { --left-bg: #1A1B26; }
.pane-right { --right-bg: #FAFAFA; }

/* Or palette-core alongside a design system that also uses --bg */
--bg: #1A1B26;         /* palette-core */
--bg: #FFFFFF;         /* collision from another library */
```

If you're the only system generating CSS variables — which is the common case — skip the prefix.

### Core — no section prefix

The 12 most-used variables. These appear in nearly every CSS rule.

| Current | Proposed | With prefix | Description |
|---------|----------|-------------|-------------|
| `--app-base-background` | `--bg` | `--app-bg` | background |
| `--app-base-background-dark` | `--bg-dark` | `--app-bg-dark` | dark/alt background |
| `--app-base-background-highlight` | `--bg-hi` | `--app-bg-hi` | highlighted background |
| `--app-base-foreground` | `--fg` | `--app-fg` | primary text |
| `--app-base-foreground-dark` | `--fg-dark` | `--app-fg-dark` | muted text |
| `--app-base-border` | `--border` | `--app-border` | border |
| `--app-base-border-highlight` | `--border-hi` | `--app-border-hi` | focused/active border |
| `--app-semantic-success` | `--success` | `--app-success` | success state |
| `--app-semantic-warning` | `--warning` | `--app-warning` | warning state |
| `--app-semantic-error` | `--error` | `--app-error` | error state |
| `--app-semantic-info` | `--info` | `--app-info` | informational |
| `--app-semantic-hint` | `--hint` | `--app-hint` | hint/subtle |

### Surfaces — `ui-`

| Current | Proposed |
|---------|----------|
| `--app-surface-menu` | `--ui-menu` |
| `--app-surface-sidebar` | `--ui-sidebar` |
| `--app-surface-statusline` | `--ui-statusline` |
| `--app-surface-float` | `--ui-float` |
| `--app-surface-popup` | `--ui-popup` |
| `--app-surface-overlay` | `--ui-overlay` |
| `--app-surface-highlight` | `--ui-hi` |
| `--app-surface-selection` | `--ui-sel` |
| `--app-surface-focus` | `--ui-focus` |
| `--app-surface-search` | `--ui-search` |

### Text — `text-`

| Current | Proposed |
|---------|----------|
| `--app-typography-comment` | `--text-comment` |
| `--app-typography-gutter` | `--text-gutter` |
| `--app-typography-line-number` | `--text-line-num` |
| `--app-typography-selection-text` | `--text-sel` |
| `--app-typography-link` | `--text-link` |
| `--app-typography-title` | `--text-title` |

### Syntax — `syn-`

| Current | Proposed |
|---------|----------|
| `--app-syntax-keywords` | `--syn-keyword` |
| `--app-syntax-keywords-fn` | `--syn-keyword-fn` |
| `--app-syntax-functions` | `--syn-fn` |
| `--app-syntax-variables` | `--syn-var` |
| `--app-syntax-variables-builtin` | `--syn-var-builtin` |
| `--app-syntax-parameters` | `--syn-param` |
| `--app-syntax-properties` | `--syn-prop` |
| `--app-syntax-types` | `--syn-type` |
| `--app-syntax-types-builtin` | `--syn-type-builtin` |
| `--app-syntax-constants` | `--syn-const` |
| `--app-syntax-numbers` | `--syn-number` |
| `--app-syntax-booleans` | `--syn-bool` |
| `--app-syntax-strings` | `--syn-string` |
| `--app-syntax-strings-doc` | `--syn-string-doc` |
| `--app-syntax-strings-escape` | `--syn-string-esc` |
| `--app-syntax-strings-regex` | `--syn-string-re` |
| `--app-syntax-operators` | `--syn-op` |
| `--app-syntax-punctuation` | `--syn-punct` |
| `--app-syntax-punctuation-bracket` | `--syn-punct-bracket` |
| `--app-syntax-annotations` | `--syn-annotation` |
| `--app-syntax-attributes` | `--syn-attr` |
| `--app-syntax-constructor` | `--syn-ctor` |
| `--app-syntax-tag` | `--syn-tag` |
| `--app-syntax-tag-delimiter` | `--syn-tag-delim` |
| `--app-syntax-tag-attribute` | `--syn-tag-attr` |
| `--app-syntax-comments` | `--syn-comment` |

### Editor — `ed-`

| Current | Proposed |
|---------|----------|
| `--app-editor-cursor` | `--ed-cursor` |
| `--app-editor-cursor-text` | `--ed-cursor-text` |
| `--app-editor-match-paren` | `--ed-match-paren` |
| `--app-editor-selection-bg` | `--ed-sel-bg` |
| `--app-editor-selection-fg` | `--ed-sel-fg` |
| `--app-editor-inlay-hint-bg` | `--ed-hint-bg` |
| `--app-editor-inlay-hint-fg` | `--ed-hint-fg` |
| `--app-editor-search-bg` | `--ed-search-bg` |
| `--app-editor-search-fg` | `--ed-search-fg` |
| `--app-editor-diagnostic-error` | `--ed-diag-error` |
| `--app-editor-diagnostic-warn` | `--ed-diag-warn` |
| `--app-editor-diagnostic-info` | `--ed-diag-info` |
| `--app-editor-diagnostic-hint` | `--ed-diag-hint` |
| `--app-editor-diagnostic-underline-error` | `--ed-diag-ul-error` |
| `--app-editor-diagnostic-underline-warn` | `--ed-diag-ul-warn` |
| `--app-editor-diagnostic-underline-info` | `--ed-diag-ul-info` |
| `--app-editor-diagnostic-underline-hint` | `--ed-diag-ul-hint` |

### Diff — `diff-`

No changes. Already concise.

| Current | Proposed |
|---------|----------|
| `--app-diff-added` | `--diff-added` |
| `--app-diff-added-bg` | `--diff-added-bg` |
| `--app-diff-added-fg` | `--diff-added-fg` |
| `--app-diff-modified` | `--diff-modified` |
| `--app-diff-modified-bg` | `--diff-modified-bg` |
| `--app-diff-modified-fg` | `--diff-modified-fg` |
| `--app-diff-removed` | `--diff-removed` |
| `--app-diff-removed-bg` | `--diff-removed-bg` |
| `--app-diff-removed-fg` | `--diff-removed-fg` |
| `--app-diff-text-bg` | `--diff-text-bg` |
| `--app-diff-ignored` | `--diff-ignored` |

### ANSI — `ansi-`

| Current | Proposed |
|---------|----------|
| `--app-terminal-black` | `--ansi-black` |
| `--app-terminal-red` | `--ansi-red` |
| `--app-terminal-green` | `--ansi-green` |
| `--app-terminal-yellow` | `--ansi-yellow` |
| `--app-terminal-blue` | `--ansi-blue` |
| `--app-terminal-magenta` | `--ansi-magenta` |
| `--app-terminal-cyan` | `--ansi-cyan` |
| `--app-terminal-white` | `--ansi-white` |
| `--app-terminal-bright-black` | `--ansi-bright-black` |
| `--app-terminal-bright-red` | `--ansi-bright-red` |
| `--app-terminal-bright-green` | `--ansi-bright-green` |
| `--app-terminal-bright-yellow` | `--ansi-bright-yellow` |
| `--app-terminal-bright-blue` | `--ansi-bright-blue` |
| `--app-terminal-bright-magenta` | `--ansi-bright-magenta` |
| `--app-terminal-bright-cyan` | `--ansi-bright-cyan` |
| `--app-terminal-bright-white` | `--ansi-bright-white` |

## Metrics

| Metric | Current | Proposed (no prefix) | Proposed (with prefix) |
|--------|---------|---------------------|----------------------|
| Median variable length | 25 chars | 14 chars | 18 chars |
| Longest variable | 41 chars | 23 chars | 27 chars |
| Core color access | `var(--app-base-background)` | `var(--bg)` | `var(--app-bg)` |
| Semantic access | `var(--app-semantic-error)` | `var(--error)` | `var(--app-error)` |

## LLM cheat sheet

The following block is designed to be pasted into an LLM system prompt. It fully defines the variable naming scheme in ~35 lines.

````
## palette-core CSS variables

Optional prefix shown in brackets. All values are hex (#RRGGBB).

### Core (no section prefix)
--[prefix-]bg              background
--[prefix-]bg-dark         dark/alt background
--[prefix-]bg-hi           highlighted background
--[prefix-]fg              foreground text
--[prefix-]fg-dark         muted text
--[prefix-]border          border
--[prefix-]border-hi       focused/active border
--[prefix-]success         success state
--[prefix-]warning         warning state
--[prefix-]error           error state
--[prefix-]info            informational
--[prefix-]hint            hint/subtle

### Surfaces: --[prefix-]ui-{name}
menu sidebar statusline float popup overlay hi sel focus search

### Text: --[prefix-]text-{name}
comment gutter line-num sel link title

### Syntax: --[prefix-]syn-{name}
keyword keyword-fn fn var var-builtin param prop type
type-builtin const number bool string string-doc string-esc
string-re op punct punct-bracket annotation attr ctor tag
tag-delim tag-attr comment

### Editor: --[prefix-]ed-{name}
cursor cursor-text match-paren sel-bg sel-fg hint-bg hint-fg
search-bg search-fg diag-error diag-warn diag-info diag-hint
diag-ul-error diag-ul-warn diag-ul-info diag-ul-hint

### Diff: --[prefix-]diff-{name}
added added-bg added-fg modified modified-bg modified-fg
removed removed-bg removed-fg text-bg ignored

### ANSI: --[prefix-]ansi-{color}
black red green yellow blue magenta cyan white
bright-black bright-red bright-green bright-yellow
bright-blue bright-magenta bright-cyan bright-white
````

## Impact on README

### Use case 1 — single preset

Before:

```css
:root {
  --app-base-background: #1A1B26;
  --app-base-foreground: #C0CAF5;
  --app-semantic-error: #DB4B4B;
}
```

After:

```css
:root {
  --bg: #1A1B26;
  --fg: #C0CAF5;
  --error: #DB4B4B;
}
```

Rust API change:

```rust
// Before
let css = palette.to_css("app");

// After
let css = palette.to_css(None);          // no prefix
let css = palette.to_css(Some("app"));   // with prefix
```

### Use case 2 — multi-theme switching

Before:

```css
body {
    background: var(--app-base-background);
    color: var(--app-base-foreground);
}
```

After:

```css
body {
    background: var(--bg);
    color: var(--fg);
}
```

No change to the `data-theme` switching mechanism.

### Use cases 3 & 4 — custom and end-user presets

No impact. TOML field names and Rust API are unchanged. Only CSS output changes.

### Typical Svelte component

Before:

```svelte
<style>
  .editor {
    background: var(--app-base-background);
    color: var(--app-base-foreground);
    border: 1px solid var(--app-base-border);
  }
  .error { color: var(--app-semantic-error); }
  .keyword { color: var(--app-syntax-keywords); }
  .comment { color: var(--app-typography-comment); }
  .diagnostic {
    border-bottom: 2px wavy var(--app-editor-diagnostic-underline-error);
  }
</style>
```

After:

```svelte
<style>
  .editor {
    background: var(--bg);
    color: var(--fg);
    border: 1px solid var(--border);
  }
  .error { color: var(--error); }
  .keyword { color: var(--syn-keyword); }
  .comment { color: var(--text-comment); }
  .diagnostic {
    border-bottom: 2px wavy var(--ed-diag-ul-error);
  }
</style>
```

## Implementation scope

**`src/css.rs`** — Replace the mechanical `{section}-{field_with_hyphens}` formatter with a mapping function that returns the short CSS name for each `(section, field)` pair. Handle optional prefix (prepend `{prefix}-` when `Some`, omit when `None`).

**`src/wasm.rs`** — Update `to_css` / `loadPresetCss` signatures to accept optional prefix.

**`src/snapshot.rs`** — No change. JSON field names continue to mirror Rust struct fields.

**Rust struct fields / TOML format** — No change.

**Tests** — Update `tests/css.rs` assertions for new variable names and optional prefix.
