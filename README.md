# christ-cli

A beautiful Bible TUI for Christian developers. Read Scripture in your terminal.

Built with Rust. Single binary. Works offline with bundled KJV.

![christ-cli demo](assets/demo.gif)

## Install

```sh
npm install -g christ-cli
```

Or with curl:
```sh
curl -fsSL https://raw.githubusercontent.com/whoisyurii/christ-cli/main/install.sh | sh
```

## Usage

Launch the interactive TUI browser:
```sh
christ
```

Read a specific verse:
```sh
christ read John 3:16
```

Read a chapter:
```sh
christ read Genesis 1
```

Read a verse range:
```sh
christ read Psalm 23:1-6
```

References work in your language too — book names in Portuguese, Spanish,
French, German, Italian, Dutch, Ukrainian, Russian, Chinese, Korean, and
Japanese are recognized, with `.` or `,` as chapter/verse separators:
```sh
christ read "João 3.16"
christ read "1. Mose 3,16"
christ read "Буття 1"
christ read "約翰福音 3:16"
```

Commands use the translation you picked in the TUI (via `v`); override it
with `-t`:
```sh
christ read "João 3.16" -t NAA
```

Search the Bible:
```sh
christ search "love one another"
```

Random verse:
```sh
christ random
```

Verse of the day:
```sh
christ today
```

Replay the startup animation:
```sh
christ intro
```

## Interactive TUI

When you run `christ` with no arguments, it launches a full-screen terminal browser:

- **Left/Right arrows / h l** - switch between panels (Books, Chapters, Scripture)
- **Up/Down arrows / j k** - navigate within a panel (moves the verse cursor in the Scripture panel)
- **Enter** - select a book or chapter
- **/** - live search the Bible (full text of the selected result shows in a preview pane)
- **y** (or **c**) - copy the selected verse to the clipboard
- **Y** (or **C**) - start a verse-range selection; move with j/k, then **Y** copies it (**Esc** cancels)
- **p** - toggle between verse-per-line and paragraph reading view (in paragraph view, **y** copies the whole chapter)
- **t** - cycle themes (Slate, Midnight, Parchment, Gospel, Terminal)
- **v** - pick a translation
- **?** - help overlay with every keybinding and its variations
- **qq** - quit (press q twice)

Your reading position, view mode, and theme are saved automatically.

## Features

- Full-screen TUI with 3-panel browser (Books | Chapters | Scripture)
- Verse-per-line or paragraph reading view, with a verse cursor
- Copy verses, ranges, or chapters to the system clipboard (native + OSC 52, so it works over SSH)
- Animated startup banner
- Live search with instant results as you type and a full-text preview
- Themes: Slate (dark), Midnight (shadcn/Vercel dark), Parchment (warm light), Gospel (bright white), Terminal (transparent)
- Bundled KJV Bible (works 100% offline, no internet required)
- Online API fallback for 50+ other translations via Bolls.life
- Forgiving reference parser (jn 3:16, 1cor 13, Ps 23:1-6, João 3.16, 1. Mose 3,16 all work)
- Book names understood in 12 languages
- Pipe-friendly (plain text when piped, rich TUI when interactive)
- Session persistence (remembers where you left off)

## Tech

- Rust single binary (~5MB)
- ratatui + crossterm for the TUI
- Bundled KJV (4.7MB embedded, public domain)
- Bolls.life API for other translations (no auth key needed)
- Cross-platform: macOS, Linux, Windows

## License

MIT
