# Worhsip Viewer

## About

Worship Viewer is a program to manage, transpose and display chord sheets.
It is developed to meet all my and hopefully your needs during a worship session from a musician's, singer's, but also technical point of view (displaying the song via beamer).
It can be used as a terminal client, tui interface or via a networked web interface.
The individual clients exchange messages via so-called websockets, e.g. when switching to the next song.
Songs can be displayed in different modes:

- Musican
- Beamer Control
- Beamer

## Installation

Currently it is unfortunately only possible to compile and install the program from source.
Also, currently only Linux is supported as a platform. However, we hope to be able to support Windows and MacOS soon.

### Build from source (Linux)

- Dependencies

  - rust-version: 1.54.0-nightly (or newer)
  - libssl-dev (openssl)
  - libncurses5-dev libncursesw5-dev (ncurses)
  - build-essential
  - npm (optional): 7.11.2 (or newer)

- Build Binary

```
  git clone https://github.com/xilefmusics/worship_viewer.git
  cd worship_viewer
  rustup override set nightly
  cargo +nightly build --release
  sudo cp ./target/release/worship_viewer /bin/wv
```

- Create recocomended folder structure

```
  mkdir ~/Songs
  mkdir ~/Songs/setlists
```

- Build Web Interface (optional)

```
  cd svelte
  npm install
  npm run build-www
  cp -r ./www ~/Songs/www
```

## Usage

The binary can be calld with different methods:

| Method                  | Functionality                                                        |
| ------------------------| -------------------------------------------------------------------- |
| wv show FILENAME        | prints a song from a songfile to stdout                              |
| wv tui ROOT_FOLDER      | starts a curses interface                                            |
| wv server ROOT_FOLDER   | starts a Server for webinterface                                     |
| wv ws_console           | starts console showing websocket messages between web clients        |
| wv import URL           | downloads song from tabs.ultimate-guitar.com and prints it to stdout |
| wv online_search TITLE  | searches for tabs.ultimate-guitar.com for song title                 |

### TUI default panel

| Key           | Action                              |
| ------------- | ----------------------------------- |
| j or space    | next song                           |
| k             | previous song                       |
| A,B,C,D,E,F,G | transpose song into key             |
| #             | transpose song into sharp variant   |
| b             | transpose song into flat variant    |
| r             | reset to default key                |
| e             | edit current song in default editor |
| t             | transpose current song              |
| /             | search                              |
| ?             | search backwards                    |
| tab           | toggle setlist selector             |
| 2             | switch to setlist editor            |
| 3             | switch to song importer             |

### TUI setlist editor

- First list: All setlists
- Middle list: Songs in current setlist
- Last list: All songs

| Key           | Action                                         |
| ------------- | ---------------------------------------------- |
| tab           | change focus between panes                     |
| j             | up                                             |
| k             | down                                           |
| space         | select setlist, remove song, add song          |
| n             | creat new setlist ( if first is list focused)  |
| d             | delete current setlist                         |
| w             | write current setlist                          |
| A,B,C,D,E,F,G | transpose song into key                        |
| #             | transpose song into sharp variant              |
| b             | transpose song into flat variant               |
| r             | reset to default key                           |
| /             | search                                         |
| ?             | search backwards                               |
| 1             | switch to default panel                        |
| 3             | switch to song importer                        |

### TUI song importer
- Imports songs from tabs.ultimate-guitar.com

| Key           | Action                              |
| ------------- | ----------------------------------- |
| s             | search online                       |
| w             | write song to library               |
| j             | down                                |
| k             | up                                  |
| /             | search in list                      |
| ?             | search backwards in list            |
| 1             | switch to default panel             |
| 2             | switch to setlist editor            |

### Web Interface

| Key                                                | Action                            |
| -------------------------------------------------- | --------------------------------- |
| j or Down or space or click on bottom of screen    | next song                         |
| k or Up or click on top of screen                  | previous song                     |
| A,B,C,D,E,F,G                                      | transpose song into key           |
| #                                                  | transpose song into sharp variant |
| b                                                  | transpose song into flat variant  |
| r                                                  | reset to default key              |
| Left or click on left of screen                    | toggle left sidebar               |
| Right or click on right of screen                  | toggle right sidebar              |
| 1                                                  | switch to musican panel           |
| 2                                                  | switch to singer panel            |
| 3                                                  | switch to beamer control panel    |
| 4                                                  | switch to beamer panel            |
| +                                                  | increment font size               |
| -                                                  | decrement font size               |
| =                                                  | reset font size                   |
| c                                                  | increment capo                    |
| x                                                  | decrement capo                    |
| v                                                  | reset capo                        |

## Formats

### Song

The song format is almost identical to the well-known ChordPro format

- A section goes always so long, until either the end of the file is reached, or a new section is defined
- After "&" a translation can be given

```
{title: Testsong}
{key: C}
{section: Intro}
[C F G C]
{section: Chorus}
{comment: This is a comment for the band}
[F]This [G]is a [C]line & Das ist eine Zeile
```

### Setlist

A setlist is a list of song titles that are assigned a key.
The song is automatically transposed to the specified key when this setlist is displayed.
Self" uses the original specified key of the song.

```
Testsong;G
OtherSong;Self
```
