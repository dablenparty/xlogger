# xlogger

The keylogger for controllers

## Platforms

- Windows
  - Developed on Windows 10/11, for Windows 10/11.
- macOS
  - Manually tested on macOS Monterey using an Intel mac.
- Linux\*
  - I have managed to get this working on Ubuntu 22.04 which is what the binary is for on the Releases page. If you are building yourself, see [Build instructions](#build-instructions).

## Installation

- Windows/MacOS:
  - Download the installer for your system and run it.
- Linux
  - For now, if you're not on Ubuntu you need to build xlogger yourself from source. See [Build instructions](#build-instructions).

## Usage

This is a GUI program can be run like any other. Previous versions of this program used a console application in the background for creating graphs of button data, but this is no longer the case. Everything is handled in the GUI.

## Features

### Current

As of the last commit to this file, `xlogger` can log data from multiple controllers at once. However, it's limited by xInput and SDL2 (I missed this in the gilrs docs before). This does _not_ include all controllers, most notably the PS5 controller. Recently, gilrs had an [update](https://gitlab.com/gilrs-project/gilrs/-/blob/master/gilrs/CHANGELOG.md#v0100-2022-11-06) that switched away from xInput on Windows; however, there is a glaring limitation. Per the linked changelog, "Apps on Windows will now require a focused window to receive inputs by default." This defeats the entire purpose of this program, so I'm forced to stick with xInput until this either changes or I can find a way around it (which is extremely unlikely).

## Build instructions

Requirements:

- Rust 1.61 stable (required by eframe)

### Linux requirements

This is not a complete list, but it's everything I had to add to get this working on Ubuntu 22.04.

- build-essential
- cmake
- fontconfig
- libglib2.0-dev
- libgtk-3-dev
- libudev-dev
- pkg-config
