# RadarSim

This is an university project.

University: Rheinische Friedrich-Wilhelms-Universität Bonn \
Module: BA-INF 137 - Einführung in die Sensordatenfusion \
Semester: SS26

## Requirements given for this project

> With "book" I am referring to "Tracking and Sensor Data Fusion" by Wolfgang Koch

- Implementation of a ground truth generator (Lecture 3, Slide 40)
- Implementation of a sensor simulator (Lecture 3, Slide 41)
- Implementation of the Kalman filter
    - Initiation (Book 3.1.4)
    - Prediction using the Piecewise Constant White Acceleration Model (Lecture 4, Slide 18f)
    - Filtering using Expectation Gates (Book 3.2.2) and Kalman Filtering (Book 3.3.1)
    - Retrodiction using Fixed Interval Smoothing (Book 3.4.1)
- Everything using 1-4 sensors and one airplane

## Installation

This project is compilable for both native and web. A hosted web version can be
found here: <https://dtiderko.github.io/radarsim>

### Dependencies

First you will need to install all dependencies. The easiest way is via [devenv](https://devenv.sh/getting-started/):

```bash
devenv shell
```

With that all required dependencies will be installed temporarily, until you exit
the shell.

If you want to install everything manually, you will at least need:
- alsa-lib
- binaryen
- clang
- libc
- libx11
- libxkbcommon
- mold
- pkg-config
- udev
- vulkan-loader
- wayland

There may be more or less dependencies but I have not tested a manual install.

### Compilation and running

As already said, you can choose between the native or web version.

#### Native

```bash
cargo build --release
./target/release/radarsim
```

or just 

```bash
cargo run --release
```

#### Web

For running this version, you will need some sort of http server. If you are using
devenv, the http server used below should already be available to you.

```bash
bash build_website.sh
http-server out/
```

