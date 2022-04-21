# xlogger

The keylogger for controllers

## Usage

For now, this is a command line program (a.k.a. CLI). Unzip it wherever you want, open PowerShell in that folder, and run `xlogger.exe`. Just use `Ctrl-C` to stop the program.

There is also a `visualize` script which is compiled from Python (yes, you read that right). When the main program receives a `Ctrl-C`, it will auto-launch the script using the data from the current session.

## Features

### Current

At the time of writing, the program is capable of logging button and stick events from a single controller. It _cannot_ differentiate between multiple controllers.

### Planned

Currently planned features are:

- A GUI to make it easier to use the program
- Visualization of the stick data
  - Most likely will draw all the points on a cartesian plane (ignoring the time) and provide some kind of way to "scrub" through the time data, updating a special point on the cartesian plane that shows the current time
- Static site generation
  - This may seem strange at first. Instead of relying on Plotly to generate the graphs on-the-fly every time, I want to generate them _once_ after each session as a static site. This means that each session can be viewed independently without having to regenerate the graphs every time.
- Better visualizations of the data (most likely through a different library entirely)
  - Chart.js is promising, as is D3
- Multiple controller support

## Build instructions

Requirements:

- Rust 1.60 stable
- Python 3.10+ (for the `visualize` script)

I have not tested this with any versions lower than those listed above. That does not mean they do not work, I just don't know if they will or not.

I highly recommend using a virtual environment for the Python part of this project. There are very few dependencies but they are quite large. With that said, once you've activated the environment, you can run `pip install -r requirements.txt` to install the dependencies and one of the build scripts (`build.bat` for dev, `build_release.bat` for release) to build the project.
