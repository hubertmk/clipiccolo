---
name: Clipiccolo
description: Big clip in, piccolo clip out. A small, warm Windows and Mac tool that turns any video into a small MP4 on your clipboard.
colors:
  espresso: "#2A1F18"
  espresso-hover: "#3D2F25"
  crema: "#F6EFE3"
  crema-desk: "#F3EDE2"
  milk-sheet: "#FFFDF9"
  oat-sheet: "#F8F2E8"
  biscotti: "#EDE5D8"
  cocoa-muted: "#6E6157"
  cocoa-faint: "#8F8276"
  hairline: "rgba(42, 31, 24, 0.11)"
  hairline-strong: "rgba(42, 31, 24, 0.22)"
  pistachio: "#8DB443"
  pistachio-ink: "#4F6E1E"
  pistachio-foam: "#E6EDD2"
  tomato: "#B3361F"
  tomato-wash: "rgba(179, 54, 31, 0.09)"
  caramel: "#8F5E12"
  caramel-wash: "rgba(143, 94, 18, 0.11)"
  roast-desk: "#17120E"
  roast-sheet: "#211A15"
  roast-oat: "#2A221C"
  roast-biscotti: "#342A23"
  roast-ink: "#F3EADF"
  roast-muted: "#B5A797"
  roast-faint: "#85786B"
  roast-hairline: "rgba(243, 234, 223, 0.10)"
  roast-hairline-strong: "rgba(243, 234, 223, 0.19)"
  roast-pistachio-ink: "#B2D46E"
  roast-pistachio: "#A6CB5E"
  roast-tomato: "#F07A62"
  roast-caramel: "#E3B341"
typography:
  display:
    fontFamily: "Bricolage Grotesque, Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(2.4rem, 5.4vw, 3.9rem)"
    fontWeight: 720
    lineHeight: 1.04
    letterSpacing: "-0.035em"
  lede:
    fontFamily: "Bricolage Grotesque, Segoe UI, system-ui, sans-serif"
    fontSize: "1.1rem"
    fontWeight: 400
    lineHeight: 1.6
  title:
    fontFamily: "Bricolage Grotesque, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    lineHeight: "20px"
  body:
    fontFamily: "Bricolage Grotesque, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: "18px"
  body-strong:
    fontFamily: "Bricolage Grotesque, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 600
    lineHeight: "18px"
  label:
    fontFamily: "Bricolage Grotesque, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: "16px"
  mono:
    fontFamily: "DM Mono, ui-monospace, Consolas, monospace"
    fontSize: "11.5px"
    fontWeight: 400
    lineHeight: "16px"
    fontFeature: "tnum"
  mono-fine:
    fontFamily: "DM Mono, ui-monospace, Consolas, monospace"
    fontSize: "0.8rem"
    fontWeight: 400
rounded:
  ctl: "7px"
  card: "10px"
  pill: "99px"
  tile: "23%"
spacing:
  segment: "2px"
  row: "10px"
  stack: "12px"
  card: "14px"
  window: "16px"
  drop: "18px"
components:
  button-primary:
    backgroundColor: "{colors.espresso}"
    textColor: "{colors.crema}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.ctl}"
    padding: "0 14px"
    height: "32px"
  button-primary-hover:
    backgroundColor: "{colors.espresso-hover}"
  button-default:
    backgroundColor: "{colors.milk-sheet}"
    textColor: "{colors.espresso}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.ctl}"
    padding: "0 14px"
    height: "32px"
  button-default-hover:
    backgroundColor: "{colors.oat-sheet}"
  button-ghost-hover:
    backgroundColor: "{colors.oat-sheet}"
    textColor: "{colors.espresso}"
  button-small:
    padding: "0 10px"
    height: "26px"
  drop-zone:
    backgroundColor: "{colors.milk-sheet}"
    textColor: "{colors.cocoa-muted}"
    rounded: "{rounded.card}"
    padding: "{spacing.drop}"
  drop-zone-over:
    backgroundColor: "{colors.pistachio-foam}"
    textColor: "{colors.espresso}"
  segment-track:
    backgroundColor: "{colors.oat-sheet}"
    rounded: "{rounded.card}"
    padding: "{spacing.segment}"
  segment-selected:
    backgroundColor: "{colors.milk-sheet}"
    textColor: "{colors.espresso}"
    rounded: "{rounded.ctl}"
    height: "28px"
  queue-card:
    backgroundColor: "{colors.milk-sheet}"
    textColor: "{colors.espresso}"
    rounded: "{rounded.card}"
    padding: "10px 12px"
  progress-fill:
    backgroundColor: "{colors.pistachio}"
    rounded: "{rounded.pill}"
    height: "4px"
  tip:
    backgroundColor: "{colors.oat-sheet}"
    textColor: "{colors.cocoa-muted}"
    typography: "{typography.label}"
    rounded: "{rounded.card}"
    padding: "10px 6px 10px 14px"
  status-error:
    backgroundColor: "{colors.tomato-wash}"
    textColor: "{colors.tomato}"
    rounded: "{rounded.card}"
    padding: "12px 14px"
  toast:
    backgroundColor: "{colors.espresso}"
    textColor: "{colors.crema-desk}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.card}"
    padding: "8px 12px"
  switch-on:
    backgroundColor: "{colors.espresso}"
    rounded: "{rounded.pill}"
    width: "34px"
    height: "20px"
---

# Design System: Clipiccolo

## Overview

**Creative North Star: "Caffè Piccolo"**

Small, strong, warm, served in seconds. Clipiccolo is built like a good espresso bar on a busy morning: the counter is warm and uncluttered, the order takes seconds, and what you're handed is small, strong and exactly right. Crema paper and espresso ink do almost all the work; pistachio is the one sweet note, kept for the moment something is done.

The app is a 380 × 520 window used in short bursts between other work, so the warmth lives in materials and details (the paper, the dot, one Italian word) and never in anything that slows the order down. Density is compact and calm: flat sheets on a paper desk, held together by hairlines, with one primary action in reach. The site keeps the same materials at a larger scale and gives the product one quiet moment of motion.

The system carries one idea from the brand mark, *due punti*: a big thing and its piccolo echo. It lives where it already belongs (the logo's two dots, the drop zone that shrinks to a strip, the site thumbnail that shrinks into its corner) and is not spread as decoration.

**Key Characteristics:**
- Crema paper, espresso ink, one sip of pistachio.
- Flat sheets on paper, held together by hairlines.
- Numbers set like a receipt, in DM Mono.
- Speed is the hospitality: no tours, no waiting screens.
- Things that finish get smaller; they don't vanish.

## Colors

A roasted, warm-neutral palette with a single green note: espresso, crema and pistachio, and a darker roast for dark mode.

### Primary
- **Espresso** (`espresso`): the ink of everything. Body text, icons that are a control's only label, the primary button fill, checked switches, text links (Copy all, Cancel, Clear list) and the toast. In dark mode its role passes to **Roast Ink** (`roast-ink`), and the primary button inverts to roast ink with an espresso label. Code: `--text`, `--action` in `ui/styles.css`.
- **Espresso Hover** (`espresso-hover`): primary button hover only.
- **Crema** (`crema`): the label on espresso buttons and the big dot of the app icon.

### Secondary
- **Pistachio** (`pistachio`): the small dot of every logo and the progress fill. Decorative only: it fails text contrast on every ground. In dark mode the fill is **Roast Pistachio** (`roast-pistachio`).
- **Pistachio Ink** (`pistachio-ink`): pistachio as text or icon, at 5.9:1 on milk sheet. Used for the success check, the "87 % smaller" saving line, the running spinner, the first-start tip icon, the drop-zone hover border and the focus ring. Dark: `roast-pistachio-ink`.
- **Pistachio Foam** (`pistachio-foam`): the drop zone while a file is dragged over it, and the text-selection highlight.

### Neutral
- **Crema Desk** (`crema-desk`): window and page background. Dark: **Roast Desk** (`roast-desk`).
- **Milk Sheet** (`milk-sheet`): cards, the drop zone, the selected segment, default buttons. Dark: `roast-sheet`.
- **Oat Sheet** (`oat-sheet`): the segmented-control track, the first-start tip, status panels and hover. Dark: `roast-oat`.
- **Biscotti** (`biscotti`): the progress track and the unchecked switch. Dark: `roast-biscotti`.
- **Cocoa Muted** (`cocoa-muted`): secondary text (subtitle, hints, descriptions, footer, default icons), 5.1:1 on crema desk. Dark: `roast-muted`.
- **Cocoa Faint** (`cocoa-faint`): placeholders and disabled text only; never the only carrier of information. Dark: `roast-faint`.
- **Hairline** / **Hairline Strong** (`hairline`, `hairline-strong`): row dividers and card borders, then control outlines, the dashed drop-zone border and the window border. Dark: `roast-hairline`, `roast-hairline-strong`.
- **Tomato** with **Tomato Wash** (`tomato`, `tomato-wash`): errors, always with an icon and a sentence; Close turns tomato on hover. **Caramel** with **Caramel Wash** (`caramel`, `caramel-wash`): warnings.

### Named Rules
**The One Sip Rule.** Pistachio covers less than 5% of any screen: progress, success and the small dot. It is never a button, a link, a heading or a fill.

**The Small Dot Rule.** Pistachio is only ever the small thing of a pair: the small dot beside the big one, the saving line under the sizes, the fill inside its track. If pistachio is the larger element, the design is wrong.

## Typography

**Display Font:** Bricolage Grotesque (with Segoe UI, system-ui)
**Body Font:** Bricolage Grotesque (with Segoe UI Variable Text, Segoe UI)
**Label/Mono Font:** DM Mono (with ui-monospace, Consolas)

**Character:** One warm grotesk does everything, with optical sizing left on, so 13 px UI text is calm and 62 px headlines get the tight display cut. DM Mono sets the numbers like a receipt. In the app both fonts ship as local woff2 in `ui/fonts/` because the app's content security policy blocks web fonts. The site self-hosts the same woff2 files in `site/fonts/` and makes no third-party requests.

### Hierarchy
- **Display** (720, clamp 2.4–3.9rem, 1.04, −0.035em): the site headline. One sentence over two lines, ending in a period; the final period may be the pistachio dot.
- **Lede** (400, 1.1rem, 1.6): the site subhead in cocoa muted, max 31rem.
- **Title** (600, 15px / 20px): the drop zone's title, the largest text in the app.
- **Body** (400, 13px / 18px): the app's base. **Body Strong** (600) for row titles, button labels and the toast.
- **Label** (400, 12px / 16px): hints, setting descriptions, the subtitle, the tip and the footer, in cocoa muted.
- **Mono** (400, 11.5px / 16px, tabular figures): file sizes, progress percentages and the version line. **Mono Fine** (0.8rem) for the fine print under the site's download button.

### Named Rules
**The Receipt Rule.** Every number people compare (sizes, percentages, dimensions, frame rates, versions, the footer's "2 done · 48.2 MB → 6.1 MB") is DM Mono with tabular figures. Sentences never are.

## Layout

The app is one frameless 380 × 520 window (minimum 340 × 440): a custom title bar, then a single column with 16px side padding and 12px between stacked blocks (drop zone, quality picker, tip or status, queue), then a footer with a hairline above it. Queue rows are padded 10 × 12 with 10px gaps; settings rows are 11 × 14 with a 52px minimum height. The queue is the only part that scrolls, with a thin scrollbar in hairline colors.

The site uses the family frame: a sticky header (lockup, four anchor stops, GitHub, one espresso Download button that steps aside while a section’s own Download button is on screen), then copy on the left and one product loop on the right in a 1.1fr / 1fr grid with a 3rem gap, stacking to one column below 760px. Below the hero, sections of 68rem width are divided by hairlines (the round trip, the preset receipt, in/out, privacy, the small things, the download card, questions) and a one-line footer closes the page. The privacy subpage reuses the same shell at a 44rem measure.

### Named Rules
**The Counter Rule.** One primary action per view, within reach. In the app that is Browse; on the site it is Download for the visitor’s own system, with the other system as an equal-sized default (milk sheet) button beside it, never a text link. If a screen needs two espresso buttons, it is two screens.

## Elevation & Depth

Flat by default. Depth comes from tonal layering (milk sheet on crema desk, oat sheet for insets) and hairlines, not shadows. The frameless window relies on the system shadow plus a 1px hairline-strong border.

### Shadow Vocabulary
- **Toast** (`box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18)`): the toast, the only thing that floats in the app.
- **Selected segment** (`box-shadow: 0 1px 2px rgba(0, 0, 0, 0.12), 0 0 0 1px` hairline): a contact shadow that lifts the chosen quality off its track.
- **Loop card** (`box-shadow: 0 1px 2px rgba(42, 31, 24, 0.06), 0 14px 28px -18px rgba(42, 31, 24, 0.45)`): the floating file card in the site's product loop.

### Named Rules
**The No Foam Rule.** Flat sheets on paper, hairlines for structure. Nothing floats except the toast and the site's loop card.

## Shapes

Gently rounded rectangles: 7px for controls (buttons, icon buttons, segments) and 10px for anything that contains (cards, the drop zone, the tip, status panels, the toast). Fully round shapes are reserved for the brand's dots, switches and progress bars. The app icon is a tile with the family's 23% corner (208 on 896), drawn from `brand/mark.svg`; at 28px in the title bar it has a 6.5px corner and, in dark mode only, a faint 1px crema ring so the espresso tile does not sink into the background. The drop zone's border is a 1.5px dash, the one dashed line in the system, marking where things can be dropped.

## Components

Warm and quiet: paper-flat, espresso where you act, pistachio only where something is done.

### Buttons
- **Shape:** gently rounded (7px), 32px tall; small buttons are 26px with 10px padding and a 12px label.
- **Primary:** espresso fill, crema label, 600 weight. One per view (Browse). Hover darkens to espresso hover.
- **Default:** milk sheet with a hairline-strong outline and an espresso label (Change…, Copy). Hover fills with oat sheet.
- **Ghost:** no fill or border until hover (Show, Retry, Default).
- **Text link:** espresso, 600, underline on hover (Copy all, Cancel, Clear list).
- **Focus:** a 2px pistachio-ink ring outside a 2px milk-sheet gap, on every interactive element.
- **Disabled:** 50% opacity.

### Inputs / Fields
- **Quality picker:** a three-way segmented control (Small, Balanced, Sharp) on an oat-sheet track with a hairline and 2px inset. The selected segment is milk sheet with an espresso label and a contact shadow; others are cocoa muted. The hint below it describes the preset in words and numbers.
- **Switch:** 34 × 20 pill; biscotti with a hairline-strong border when off, espresso when on, with a crema knob that slides 14px in 120ms.

### Cards / Containers
- **Corner Style:** 10px.
- **Background:** milk sheet on crema desk.
- **Shadow Strategy:** none (see Elevation & Depth).
- **Border:** 1px hairline; rows inside are divided by hairlines, never by gaps.
- **Internal Padding:** 10 × 12 in the queue, 11 × 14 in settings.

### Drop Zone (signature)
The large target where videos come in: a milk-sheet field with a 1.5px dashed hairline-strong border, 176px tall with an 18px pad, centered icon, title, hint and the Browse button. Once the queue has files it becomes a one-line strip (10 × 12 padding, hint hidden). While a file is dragged over it, the border turns solid pistachio ink on pistachio foam.

### Queue Row
A row per video with a 16px state icon, the name (body strong, ellipsized) and a mono line of sizes. Running rows add a 4px pistachio progress fill on a biscotti track, always with the percentage. Done rows put the saving on its own pistachio-ink line and offer Copy and Show; failed rows show the error sentence in tomato and offer Retry.

### First-start Tip
Shown once, on the very first start, until the first video arrives or it is dismissed: an oat-sheet panel with a 10px corner, a pistachio-ink clipboard icon, one sentence in label type, and a 24px dismiss button. It teaches the clipboard path once and never returns.

### Status and Toast
- **Status:** an oat-sheet panel for one ongoing task (pistachio-ink spinner), or a tomato-wash panel for an error (tomato icon). The title says what happened; the text says what to do next and is selectable.
- **Toast:** an espresso card (10px corner) with crema-desk text and the toast shadow, rising 6px into place and fading in over 160ms above the footer, gone after 2.6s. It holds the app's one Italian word: *Fatto. Copied to clipboard.*

### Named Rules
**The Piccolo Echo Rule.** When something completes or collapses, it gets smaller instead of disappearing: the drop zone becomes a strip when work starts, the first-start tip shrinks away (200ms) when dismissed, and the site's thumbnail shrinks into its corner when a clip is done. Prefer shrinking to hiding when a state ends.

### Brand Surfaces
Everything outside the window carries the same materials, generated by `brand/build.py` from one geometry source:
- **Installer:** the NSIS sidebar (164 × 314: crema desk, tile, wordmark, the headline with its pistachio period) and header (150 × 57: the tile on the installer's white band).
- **License page:** `THIRD-PARTY-LICENSES.html` (template `about.hbs`) uses the same tokens in both themes, milk-sheet sections and a pistachio period after its title.
- **Link preview:** `brand/png/social.png` and `site/og.png` (1280 × 640): lockup, headline, fine print in DM Mono, and the mark's two dots on the headline's baseline.
- **README:** the lockup, switching to `lockup-dark.svg` on GitHub's dark theme.

## Do's and Don'ts

### Do:
- **Do** use espresso (#2A1F18) for the single primary action and crema (#F6EFE3) for its label.
- **Do** confirm routine actions with a past-tense toast and no color; keep pistachio for finished videos.
- **Do** set every size, percentage and version in DM Mono with tabular figures.
- **Do** draw the brand from `brand/` (run `python brand/build.py`); never redraw the mark or set the wordmark in live type. Change installer art and the link preview in `build.py`, not in the output files.
- **Do** keep both themes in step: every light token has its roast counterpart, and text pairs stay at WCAG AA in both.

### Don't:
- **Don't** use pistachio for buttons, links, headings, backgrounds or any large fill.
- **Don't** draw cups, beans, steam or café scenes. The metaphor lives in materials, never in pictures.
- **Don't** add a loading, welcome or onboarding screen; the drop zone is the welcome and the tip appears once.
- **Don't** scatter dots as bullets or decoration. Dots come as the brand's meaningful pair or not at all.
- **Don't** make buttons round or pill-shaped to echo the dots. Controls stay 7px rectangles.
- **Don't** add shadows to cards, rows or the drop zone.
