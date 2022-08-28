# xlogger

The keylogger for controllers

## Platforms

Below are the platforms I have tested xlogger on:

- Windows
  - Developed on Windows 10/11, for Windows 10/11.
- MacOS?
  - Manually tested on MacOS Monterey, installed via dmg.
- Linux\*
  - I have managed to get this working on Ubuntu 22.04. If you are building yourself, see [Build instructions](#build-instructions).

## Installation

- Windows/MacOS:
  - Download the installer for your system and run it.
- Linux
  - For now, you need to build xlogger yourself from source. See [Build instructions](#build-instructions).

## Usage

This is a GUI program can be run like any other. Previous versions of this program used a console application in the background for creating graphs of button data, but this is no longer the case. Everything is handled in the GUI.

## Features

### Current

As of the last commit to this file, `xlogger` can log data from multiple controllers at once. However, it's limited by XInput and SDL2 (I missed this in the gilrs docs before). This does _not_ include all controllers, most notably the PS5 controller.

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
