# UI/UX Design Rules

This document defines mandatory UI/UX patterns and conventions for this project.

## 1. Icon & Visual Elements

### NO Emoji Icons (CRITICAL)

| Rule | Do | Don't |
|------|----|----- |
| **No emoji icons** | Use SVG icons (Heroicons, Lucide, Simple Icons) | Use emojis like 🎨 🚀 ⚙️ as UI icons |
| **Consistent icon set** | Use ONE icon library consistently | Mix Heroicons with FontAwesome with Lucide |
| **Icon sizing** | Use fixed viewBox (24x24) with consistent sizing | Mix different icon sizes randomly |

### Brand Logos

- **ALWAYS** verify brand logos from official sources (Simple Icons)
- **NEVER** guess logo paths or use incorrect SVGs
- Use official brand colors when applicable

## 2. Interaction Patterns

### Cursor States

```css
/* ✅ GOOD: Clear interactive indicators */
.clickable {
  cursor: pointer;
}

.disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

/* ❌ BAD: No cursor indication on interactive elements */
.button {
  /* missing cursor: pointer */
}
```

### Hover & Focus States

| Element | Required States |
|---------|-----------------|
| Buttons | `:hover`, `:focus`, `:active`, `:disabled` |
| Links | `:hover`, `:focus`, `:visited` |
| Cards | `:hover` (if clickable), `:focus-within` |
| Inputs | `:focus`, `:invalid`, `:disabled` |

### Transitions

```css
/* ✅ GOOD: Smooth, appropriate transitions */
.element {
  transition: color 200ms ease, background-color 200ms ease;
}

/* ❌ BAD: Too slow or instant */
.element {
  transition: all 800ms; /* Too slow */
}
.element {
  /* No transition - jarring */
}
```

**Recommended Duration**: 150-300ms for most interactions

### Layout Stability

- **NEVER** use `transform: scale()` on hover if it causes layout shift
- Use `transform` with proper spacing/padding to prevent clipping
- Prefer `color`, `background-color`, `box-shadow` changes

## 3. Light/Dark Mode

### Contrast Requirements (WCAG AA)

| Element | Light Mode | Dark Mode |
|---------|------------|-----------|
| Body text | #0F172A (slate-900) minimum | #F8FAFC (slate-50) minimum |
| Muted text | #475569 (slate-600) minimum | #94A3B8 (slate-400) minimum |
| Borders | `border-gray-200` | `border-gray-700` |

### Glass/Transparent Elements

```css
/* ✅ GOOD: Visible in light mode */
.glass-card {
  background: rgba(255, 255, 255, 0.8); /* Light mode */
}

.dark .glass-card {
  background: rgba(15, 23, 42, 0.8); /* Dark mode */
}

/* ❌ BAD: Too transparent in light mode */
.glass-card {
  background: rgba(255, 255, 255, 0.1); /* Invisible */
}
```

## 4. Layout & Spacing

### Container Consistency

- Use ONE consistent `max-width` throughout the app
- Recommended: `max-w-6xl` or `max-w-7xl`
- Center with `mx-auto`

### Fixed Element Spacing

```css
/* ✅ GOOD: Floating navbar with spacing */
.navbar {
  position: fixed;
  top: 1rem;    /* Not 0 */
  left: 1rem;
  right: 1rem;
}

/* Account for navbar in content */
.main-content {
  padding-top: 5rem; /* navbar height + spacing */
}
```

### Responsive Breakpoints

Test at these widths:
- 320px (small mobile)
- 768px (tablet)
- 1024px (desktop)
- 1440px (large desktop)

**RULE**: No horizontal scroll on any breakpoint

## 5. Typography

### Font Pairing

- Use maximum 2-3 font families
- Headings: Display/Serif font
- Body: Sans-serif font
- Code: Monospace font

### Font Loading

```html
<!-- ✅ GOOD: Proper font loading -->
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
```

### Size Scale

Use consistent size scale (e.g., Tailwind defaults):
- `text-xs`: 12px
- `text-sm`: 14px
- `text-base`: 16px
- `text-lg`: 18px
- `text-xl`: 20px
- `text-2xl`: 24px
- etc.

## 6. Color Usage

### Semantic Colors

| Purpose | Token | Usage |
|---------|-------|-------|
| Primary | `--primary` | Main CTAs, brand elements |
| Secondary | `--secondary` | Supporting actions |
| Accent | `--accent` | Highlights, badges |
| Destructive | `--destructive` | Delete, error states |
| Muted | `--muted` | Backgrounds, borders |

### Color Accessibility

- Never use color alone to convey information
- Combine with text, icons, or patterns
- Maintain 4.5:1 contrast ratio for text

## 7. Animation Guidelines

### Performance Rules

```css
/* ✅ GOOD: GPU-accelerated properties */
.animate {
  transform: translateX(0);
  opacity: 1;
}

/* ❌ BAD: Layout-triggering properties */
.animate {
  left: 0;      /* Causes layout */
  width: 100px; /* Causes layout */
}
```

### Motion Preferences

```css
/* ✅ GOOD: Respect user preferences */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

## 8. Accessibility Checklist

### Images

- [ ] All `<img>` have `alt` attribute
- [ ] Decorative images use `alt=""`
- [ ] Complex images have longer descriptions

### Forms

- [ ] All inputs have associated `<label>`
- [ ] Error messages are descriptive
- [ ] Required fields are indicated

### Navigation

- [ ] Skip link to main content
- [ ] Logical heading hierarchy (h1 → h2 → h3)
- [ ] Focus visible on all interactive elements

### ARIA

- [ ] ARIA used correctly (prefer semantic HTML first)
- [ ] Live regions for dynamic content
- [ ] Modals trap focus correctly

## 9. Pre-Delivery Checklist

Before delivering ANY UI code, verify:

### Visual Quality
- [ ] No emojis used as icons (use SVG instead)
- [ ] All icons from consistent icon set
- [ ] Hover states don't cause layout shift
- [ ] Theme colors used directly (not var() wrapper in Tailwind)

### Interaction
- [ ] All clickable elements have `cursor-pointer`
- [ ] Hover states provide clear visual feedback
- [ ] Transitions are smooth (150-300ms)
- [ ] Focus states visible for keyboard navigation

### Light/Dark Mode
- [ ] Light mode text has sufficient contrast
- [ ] Glass/transparent elements visible in light mode
- [ ] Borders visible in both modes
- [ ] Test both modes before delivery

### Layout
- [ ] Floating elements have proper edge spacing
- [ ] No content hidden behind fixed navbars
- [ ] Responsive at 320px, 768px, 1024px, 1440px
- [ ] No horizontal scroll on mobile

### Accessibility
- [ ] All images have alt text
- [ ] Form inputs have labels
- [ ] Color is not the only indicator
- [ ] `prefers-reduced-motion` respected
