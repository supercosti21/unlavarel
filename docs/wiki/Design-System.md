# Design System

## Overview

Unlavarel uses a custom CSS design system inspired by [Linear.app](https://linear.app) — minimal, dense but clean, dark-first.

**No Tailwind.** All styling uses CSS custom properties defined in `src/app.css`.

## Principles

1. **Minimal** — No heavy borders, gradients, or excessive shadows
2. **Dense** — Efficient use of space, information-rich UI
3. **Dark-first** — Dark theme is default, light theme available
4. **Consistent** — All components use the same design tokens
5. **System fonts** — Native system-ui for UI, native mono for code

## Color Palette

### Dark Theme (Default)

| Token | Value | Usage |
|-------|-------|-------|
| `--color-bg-primary` | `#0a0a0f` | App background |
| `--color-bg-secondary` | `#12121a` | Sidebar, statusbar |
| `--color-bg-card` | `#16161f` | Card surfaces |
| `--color-bg-hover` | `#1e1e2a` | Hover states |
| `--color-border` | `#2a2a3a` | Primary borders |
| `--color-border-subtle` | `#1f1f2f` | Subtle separators |
| `--color-text-primary` | `#e8e8ed` | Main text |
| `--color-text-secondary` | `#8888a0` | Secondary text |
| `--color-text-muted` | `#555570` | Disabled/hint text |

### Accent Colors

| Token | Value | Usage |
|-------|-------|-------|
| `--color-accent` | `#8b5cf6` | Primary actions, active nav |
| `--color-accent-hover` | `#7c3aed` | Hover on accent buttons |
| `--color-success` | `#22c55e` | Running, success states |
| `--color-danger` | `#ef4444` | Stopped, error states |
| `--color-warning` | `#eab308` | Warning, transitioning |

### Light Theme

Activated via `data-theme="light"` on the `<html>` element. Overrides all background and text colors with light variants.

## Typography

```css
--font-sans: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
--font-mono: 'SF Mono', 'Cascadia Code', 'JetBrains Mono', 'Fira Code', monospace;
```

### Scale

| Token | Size | Usage |
|-------|------|-------|
| `--text-xs` | 0.75rem (12px) | Badges, status, metadata |
| `--text-sm` | 0.875rem (14px) | Default body text |
| `--text-base` | 1rem (16px) | Section headings |
| `--text-lg` | 1.125rem (18px) | Page sub-headings |
| `--text-xl` | 1.25rem (20px) | Page titles |
| `--text-2xl` | 1.5rem (24px) | Hero text |

## Spacing

```css
--space-1: 0.25rem;   /*  4px */
--space-2: 0.5rem;    /*  8px */
--space-3: 0.75rem;   /* 12px */
--space-4: 1rem;      /* 16px */
--space-6: 1.5rem;    /* 24px */
--space-8: 2rem;      /* 32px */
```

## Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 6px | Buttons, inputs, small elements |
| `--radius-md` | 8px | Standard containers |
| `--radius-lg` | 12px | Cards, dialogs |
| `--radius-full` | 9999px | Badges, dots |

## Components

### Status Dot
```css
.status-dot--running  → green with glow
.status-dot--stopped  → red, no glow
.status-dot--error    → yellow
.status-dot--unknown  → gray
```

### Buttons
- `.btn-primary` — Purple accent, white text
- `.btn-ghost` — Transparent, shows on hover
- `.btn-danger` — Red subtle background, red text → solid red on hover

### Badges
- `.badge--success` — Green subtle background
- `.badge--danger` — Red subtle background
- `.badge--warning` — Yellow subtle background
- `.badge--neutral` — Gray background

### Cards
```css
.card {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-5);
  box-shadow: var(--shadow-card);
}
```

## Layout

The app uses a fixed sidebar + scrollable main area:

```
┌──────────┬──────────────────────────────┐
│          │                              │
│ Sidebar  │        Main Content          │
│  220px   │     (scrollable)             │
│          │                              │
│          │                              │
├──────────┴──────────────────────────────┤
│              Status Bar (32px)           │
└─────────────────────────────────────────┘
```
