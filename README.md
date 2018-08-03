Rune
====

Rune is a tool to post-process Haiku raw ARM images for various target devices.

[![Build Status](https://travis-ci.org/kallisti5/rune-image.svg?branch=master)](https://travis-ci.org/kallisti5/rune-image) [![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Features
---------

  * Coordinates with a [remote manifest](https://github.com/haiku/firmware/blob/master/manifest.json) at Github of known target boards.
  * Injects any needed vendor specific boot binaries from remote sources.
  * Writes directly to an SD card, or to a new image file.

Why is this needed?
----

The ARM ecosystem contains a wide range of technology and boot processes. While
this variability has been great for innovation, it also means operating systems
need to be custom tailored per device.

Specific boot files on SD cards, secondary loaders at specific offsets,
binary vendor blobs tailored just right per SOC specifications and GPL licensed
u-boot binaries make the ARM ecosystem a tricky beast to conquer.

Why Rust?
---------

  * Low level enough to directly write to files and make modifications without relying on external tools.
  * Cross-platform. Rune is designed to be used by end-users across multiple operating systems.
  * Easy json parsing and HTTP GET's without requiring a large number of libraries.

Example Usage
-------------

  * **Listing available boards:** ```rune -l```
  * **Prepare an SD card for the Raspberry Pi 2:** ```rune -b rpi2 -i haiku-arm.mmc /dev/sdc```
  * **Write the prepared disk image to a file:** ```rune -b rpi2 -i haiku-arm.mmc /home/alex/haiku-arm-rpi2.mmc```
  * **Make an SD card bootable which has had Haiku dd'ed to it:** ```rune -b rpi2 /dev/sdc```

Compiling
---------

  * Install rust 1.26.0 or later. https://rustup.rs/ can be used if your distro doesn't provide.
  * cargo build --release

Credit
------

  * Thanks to Fedora for creating fedora-arm-installer which was the inspiration for this tool.
  * Thanks to the great Rust folks for helping out!
