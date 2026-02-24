# CSS variables reference

`palette.to_css(prefix)` exports up to 95 CSS custom properties. The prefix is optional — omit it for clean names, or pass one to avoid collisions with other variable systems on the same page.

```rust
let css = palette.to_css(None);          // --bg, --fg, --error
let css = palette.to_css(Some("app"));   // --app-bg, --app-fg, --app-error
```

## When to use a prefix

Skip it if palette-core is the only system generating CSS variables — the common case.

Use a prefix when another library also defines bare names like `--bg` or `--error`, or when multiple palette-core instances coexist on the same page:

```css
/* Two palette-core instances (e.g. split-pane editor) */
.pane-left  { --left-bg: #1A1B26; }
.pane-right { --right-bg: #FAFAFA; }
```

## Variable names

Optional prefix shown in brackets. All values are hex (`#RRGGBB`).

### Core (no section prefix)

```
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
```

### Surfaces: `--[prefix-]ui-{name}`

```
menu sidebar statusline float popup overlay hi sel focus search
```

### Text: `--[prefix-]text-{name}`

```
comment gutter line-num sel link title
```

### Syntax: `--[prefix-]syn-{name}`

```
keyword keyword-fn fn var var-builtin param prop type
type-builtin const number bool string string-doc string-esc
string-re op punct punct-bracket annotation attr ctor tag
tag-delim tag-attr comment
```

### Editor: `--[prefix-]ed-{name}`

```
cursor cursor-text match-paren sel-bg sel-fg hint-bg hint-fg
search-bg search-fg diag-error diag-warn diag-info diag-hint
diag-ul-error diag-ul-warn diag-ul-info diag-ul-hint
```

### Diff: `--[prefix-]diff-{name}`

```
added added-bg added-fg modified modified-bg modified-fg
removed removed-bg removed-fg text-bg ignored
```

### ANSI: `--[prefix-]ansi-{color}`

```
black red green yellow blue magenta cyan white
bright-black bright-red bright-green bright-yellow
bright-blue bright-magenta bright-cyan bright-white
```

## Usage example

```css
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
```
