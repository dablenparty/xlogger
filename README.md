# xlogger

The keylogger for controllers

## Platforms

Below are the platforms I have tested xlogger on:

- [x] Windows
  - Developed on Windows 10/11, for Windows 10/11.
- [ ] MacOS?
  - It builds, but I haven't fully tested it so I can't say if it works or not.
- [ ] ~~Linux~~
  - I haven't forgotten about Linux gamers, but I have not had much luck getting `xlogger` to even build on this platform. Eventually, I want this to be fully cross-platform, but until I have more time to test on Linux, this will remain unsupported.

## Installation

- Download the latest release zip file
- Extract it wherever you want (I highly recommend extracting to its own folder)
- Create a shortcut for `xlogger.exe` and put it wherever (optional)
- Run it

## Usage

This is a GUI program can be run like any other. Previous versions of this program used a console application in the background for creating graphs of button data, but this is no longer the case. Everything is handled in the GUI.

## Features

### Current

As of the last commit to this file, `xlogger` _can_ differentiate between multiple controllers, although it does not currently support anything other than XInput (hence the name xlogger). This is, however, a work in progress. What this means is you can use the program to log data from multiple controllers at once.

### Planned

Currently planned features are:

- Wider controller support
  - Expanding support to controllers that don't use XInput (e.g., PS4/5 controllers) via SDL

## Build instructions

Requirements:

- Rust 1.61 stable (required by eframe)
