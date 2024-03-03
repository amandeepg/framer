# Framer
Framer will composite a screenshot onto a device frame, by intelligently finding the space for the screenshot.

Built in Rust.

### Usage
```
Usage: framer [OPTIONS] <DEVICE_FRAME_PATH> <SCREENSHOT_PATH>

Arguments:
  <DEVICE_FRAME_PATH>  Path to device frame image
  <SCREENSHOT_PATH>    Path to screenshot image

Options:
  -o <OUTPUT_PATH>                  Path to composited output image [default: ./result.png]
  -t, --top-search-axis <percent>   How far, as a percentage, from the left edge to search for the top edge upwards and bottom edge downwards. For example if there is a notch, the default of 25 may hit the notch rather than the top of the frame [default: 25]
  -l, --left-search-axis <percent>  How far, as a percentage, from the top edge to search for the left edge leftwards and the right edge rightwards [default: 50]
      --oxipng-level <level>        The level of optimization to use with oxipng (0-6), lower is faster [default: 4]
      --pngquant-speed <speed>      The level of optimization to use with pngquant (1-10), higher is faster [default: 4]
  -h, --help                        Print help
  -V, --version                     Print version

```

### Example
Take the following 2 files as input, the frame with marketing text, and the screenshot, produces the 3rd image, the composited image.

# <img src="./docs/frame1.webp" height="300"/> &nbsp;**+**&nbsp; <img src="./docs/screenshot1.webp" height="300"/> &nbsp;&nbsp;&nbsp;**=**&nbsp;&nbsp;&nbsp; <img src="./docs/framescr1.webp" height="300"/>
