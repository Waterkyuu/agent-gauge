# Ollama — Style Reference
> An almost defiantly minimal documentation-first system that treats the home page like a Markdown README — paper-white canvas, 36px center-aligned heading, a single black pill CTA, an inline terminal install snippet, and a hand-drawn llama mascot as the only ornamental element. No gradient, no hero photography, no marketing pyrotechnics. The chrome is a tiny utility palette of pure black, pure white, and three neutral grays; every interactive element is fully rounded into a pill (`{rounded.full}`); typography is SF Pro Rounded for headings paired with system sans for body and ui-monospace for code. Pricing tiers, FAQs, and "your data stays yours" guarantees all sit on the same flat canvas inside thin-border cards — the system is the documentation, and the documentation is the system.

**Theme:** light

**Source website:** [https://ollama.com/](https://ollama.com/)  
Use the live official website to compare and validate this extracted snapshot. The current source website remains authoritative.

## Tokens — Colors

| Name | Value | Token | Role |
|---|---|---|---|
| primary | `#000000` | `--color-primary` | primary role extracted from the source design |
| on primary | `#ffffff` | `--color-on-primary` | on primary role extracted from the source design |
| ink | `#000000` | `--color-ink` | ink role extracted from the source design |
| ink deep | `#090909` | `--color-ink-deep` | ink deep role extracted from the source design |
| charcoal | `#525252` | `--color-charcoal` | charcoal role extracted from the source design |
| body | `#737373` | `--color-body` | body role extracted from the source design |
| mute | `#a3a3a3` | `--color-mute` | mute role extracted from the source design |
| canvas | `#ffffff` | `--color-canvas` | canvas role extracted from the source design |
| surface soft | `#fafafa` | `--color-surface-soft` | surface soft role extracted from the source design |
| surface card | `#ffffff` | `--color-surface-card` | surface card role extracted from the source design |
| hairline | `#e5e5e5` | `--color-hairline` | hairline role extracted from the source design |
| hairline strong | `#d4d4d4` | `--color-hairline-strong` | hairline strong role extracted from the source design |
| on dark | `#ffffff` | `--color-on-dark` | on dark role extracted from the source design |
| on dark mute | `rgba(255,255,255,0.7)` | `--color-on-dark-mute` | on dark mute role extracted from the source design |
| surface dark | `#171717` | `--color-surface-dark` | surface dark role extracted from the source design |
| focus ring | `rgba(59,130,246,0.5)` | `--color-focus-ring` | focus ring role extracted from the source design |
| link | `#000000` | `--color-link` | link role extracted from the source design |
| link mute | `#737373` | `--color-link-mute` | link mute role extracted from the source design |
| terminal red | `#ff5f56` | `--color-terminal-red` | terminal red role extracted from the source design |
| terminal yellow | `#ffbd2e` | `--color-terminal-yellow` | terminal yellow role extracted from the source design |
| terminal green | `#27c93f` | `--color-terminal-green` | terminal green role extracted from the source design |

## Tokens — Typography

### SF Pro Rounded · `--font-primary`
- **Substitute:** Inter, system-ui, sans-serif
- **Weights:** 500, 600
- **Sizes:** 36px, 30px, 24px
- **Line height:** 1.11, 1.2, 1.33
- **Letter spacing:** 0
- **Role:** Brand typography family observed across the documented type scale.

### ui-sans-serif · `--font-family-2`
- **Substitute:** Inter, system-ui, sans-serif
- **Weights:** 500, 400
- **Sizes:** 20px, 18px, 16px, 14px, 12px
- **Line height:** 1.4, 1.56, 1.5, 1.43, 1.33, 1
- **Letter spacing:** 0
- **Role:** Brand typography family observed across the documented type scale.

### ui-monospace · `--font-family-3`
- **Substitute:** Inter, system-ui, sans-serif
- **Weights:** 400
- **Sizes:** 16px, 14px
- **Line height:** 1.5, 1.43
- **Letter spacing:** 0
- **Role:** Brand typography family observed across the documented type scale.

### Type Scale

| Role | Size | Line Height | Letter Spacing | Token |
|---|---|---|---|---|
| display-xl | 36px | 1.11 | 0 | `--text-display-xl` |
| display-lg | 30px | 1.2 | 0 | `--text-display-lg` |
| heading-lg | 24px | 1.33 | 0 | `--text-heading-lg` |
| heading-md | 20px | 1.4 | 0 | `--text-heading-md` |
| heading-sm | 18px | 1.56 | 0 | `--text-heading-sm` |
| body-md | 16px | 1.5 | 0 | `--text-body-md` |
| body-strong | 16px | 1.5 | 0 | `--text-body-strong` |
| body-sm | 14px | 1.43 | 0 | `--text-body-sm` |
| body-sm-strong | 14px | 1.43 | 0 | `--text-body-sm-strong` |
| caption-sm | 12px | 1.33 | 0 | `--text-caption-sm` |
| code-md | 16px | 1.5 | 0 | `--text-code-md` |
| code-sm | 14px | 1.43 | 0 | `--text-code-sm` |
| button-md | 14px | 1 | 0 | `--text-button-md` |

## Tokens — Spacing & Shapes

**Density:** comfortable

### Spacing Scale

| Name | Value | Token |
|---|---|---|
| xxs | 2px | `--spacing-xxs` |
| xs | 4px | `--spacing-xs` |
| sm | 8px | `--spacing-sm` |
| md | 12px | `--spacing-md` |
| lg | 16px | `--spacing-lg` |
| xl | 24px | `--spacing-xl` |
| xxl | 32px | `--spacing-xxl` |
| section | 88px | `--spacing-section` |

### Border Radius

| Name | Value | Token |
|---|---|---|
| none | 0px | `--radius-none` |
| sm | 6px | `--radius-sm` |
| md | 8px | `--radius-md` |
| lg | 12px | `--radius-lg` |
| full | 9999px | `--radius-full` |

### Layout

- **Section gap:** 88px
- **Card padding:** 16px
- **Element gap:** 12px
- **Max content width:** 1200px

## Components

### button primary
**Role:** button primary component

- **backgroundColor:** `{colors.primary}`
- **textColor:** `{colors.on-primary}`
- **typography:** `{typography.button-md}`
- **rounded:** `{rounded.full}`
- **padding:** `8px 20px`
- **height:** `36px`

### button primary active
**Role:** button primary active component

- **backgroundColor:** `{colors.ink-deep}`
- **textColor:** `{colors.on-primary}`
- **typography:** `{typography.button-md}`
- **rounded:** `{rounded.full}`

### button secondary
**Role:** button secondary component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.button-md}`
- **rounded:** `{rounded.full}`
- **padding:** `8px 20px`
- **height:** `36px`

### button pill on dark
**Role:** button pill on dark component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.button-md}`
- **rounded:** `{rounded.full}`
- **padding:** `8px 20px`

### button disabled
**Role:** button disabled component

- **backgroundColor:** `{colors.surface-soft}`
- **textColor:** `{colors.mute}`
- **rounded:** `{rounded.full}`

### search pill
**Role:** search pill component

- **backgroundColor:** `{colors.surface-soft}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.body-sm}`
- **rounded:** `{rounded.full}`
- **padding:** `8px 16px`
- **height:** `36px`

### search pill focused
**Role:** search pill focused component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.ink}`
- **rounded:** `{rounded.full}`

### text input
**Role:** text input component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.body-md}`
- **rounded:** `{rounded.full}`
- **padding:** `8px 16px`
- **height:** `40px`

### text input focused
**Role:** text input focused component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.ink}`
- **rounded:** `{rounded.full}`

### install snippet
**Role:** install snippet component

- **backgroundColor:** `{colors.surface-soft}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.code-md}`
- **rounded:** `{rounded.full}`
- **padding:** `12px 20px`
- **height:** `48px`

### command tag
**Role:** command tag component

- **backgroundColor:** `{colors.surface-soft}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.code-sm}`
- **rounded:** `{rounded.full}`
- **padding:** `6px 12px`

### terminal card
**Role:** terminal card component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.code-sm}`
- **rounded:** `{rounded.lg}`
- **padding:** `16px`

### terminal traffic lights
**Role:** terminal traffic lights component

- **rounded:** `{rounded.full}`
- **size:** `12px`

### pricing card
**Role:** pricing card component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.body-md}`
- **rounded:** `{rounded.lg}`
- **padding:** `32px`

### pricing card dark
**Role:** pricing card dark component

- **backgroundColor:** `{colors.surface-dark}`
- **textColor:** `{colors.on-dark}`
- **typography:** `{typography.body-md}`
- **rounded:** `{rounded.lg}`
- **padding:** `32px`

### feature bullet
**Role:** feature bullet component

- **textColor:** `{colors.charcoal}`
- **typography:** `{typography.body-sm}`

### faq row
**Role:** faq row component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.body-md}`
- **rounded:** `{rounded.none}`
- **padding:** `16px 0px`

### link inline
**Role:** link inline component

- **textColor:** `{colors.ink}`
- **typography:** `{typography.body-md}`

### link mute
**Role:** link mute component

- **textColor:** `{colors.body}`
- **typography:** `{typography.body-sm}`

### primary nav
**Role:** primary nav component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.ink}`
- **typography:** `{typography.body-sm-strong}`
- **rounded:** `{rounded.none}`
- **height:** `56px`

### footer section
**Role:** footer section component

- **backgroundColor:** `{colors.canvas}`
- **textColor:** `{colors.body}`
- **typography:** `{typography.caption-sm}`
- **rounded:** `{rounded.none}`
- **padding:** `32px 24px`

### cta strip dark
**Role:** cta strip dark component

- **backgroundColor:** `{colors.surface-dark}`
- **textColor:** `{colors.on-dark}`
- **typography:** `{typography.heading-lg}`
- **rounded:** `{rounded.lg}`
- **padding:** `24px 32px`

## Do's and Don'ts

### Do

- Use `--color-primary` for the brand's primary interaction treatment.
- Keep page surfaces anchored to `--color-canvas`.
- Preserve every typography style's documented size, line height, and letter spacing.
- Compare major implementation decisions against [the live Ollama website](https://ollama.com/).

### Don't

- Do not introduce colors outside the documented color token set.
- Do not replace `--color-ink` with an arbitrary neutral.
- Do not flatten documented component states or spacing relationships.
- Do not treat this extracted snapshot as newer than the live source website.

## Layout

Use the documented spacing scale and component geometry as the implementation baseline. Validate responsive composition and current page rhythm against [the live source](https://ollama.com/).
