# xlogger

The keylogger for controllers

## Installation

- Download the latest release zip file
- Extract it wherever you want (I highly recommend extracting to its own folder)
- Create a shortcut for `xlogger.exe` and put it wherever (optional)
- Run it

## Usage

This is a GUI program can be run like any other; however, the executable (a.k.a. the `xlogger.exe` file) MUST stay in the same folder as the `visualize` folder it comes with.

## Features

### Current

At the time of writing, the program is capable of logging button and stick events from a single controller. It can visualize both stick and button data. It _cannot_ differentiate between multiple controllers, nor does it currently support anything other than XInput (hence the name xlogger).

### Planned

Currently planned features are:

- Multiple controller support
  - Connecting and logging multiple controllers at once
- Wider controller support
  - Expanding support to controllers that don't use XInput (e.g., PS4/5 controllers)
- "In-house" button data visualization
  - Currently, the button data is created

## Build instructions

Requirements:

- Rust 1.60 stable
- Python 3.10+ (for the `visualize` script)

I have not tested this with any versions lower than those listed above. That does not mean they do not work, I just don't know if they will or not.

I highly recommend using a virtual environment for the Python part of this project. There are very few dependencies but they are quite large. With that said, once you've activated the environment, you can run `pip install -r requirements.txt` to install the dependencies and one of the build scripts (`build.bat` for dev, `build_release.bat` for release) to build the project.
