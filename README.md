# resvg Private Fork
This is Shaper Tools fork of the resvg library.

## Development Setup
**Shaper development should be on develop branch only.**

Master branch is sync'd with public repo at (https://github.com/RazrFalcon/resvg/)

To add public library as upstream remote:

`git remote add upstream git@github.com:RazrFalcon/resvg.git`

`git remote set-url --push upstream DISABLE`


## Project Structure

```
resvg (Converts svg to bitmaps)
|
|
|____usvg (CLI utility and library for converting SVG into uSvg data)
|
|
|____bullet_svg (Shaper library wrapper around usvg)
|
|
|____bullet_wasm (Shaper wasm library wrapper and js bindings around bullet-svg for use on web)
|   |
|   |
|   |____www (Demo and test web page for bullet_wasm library and js bindings)
|   |
|   |
|   |____pkg (Compiled wasm and js binding files, and package.json file for distributing via npm)

```

### Parser Architecture
```
usvg/lib.rs (Converts svg to usvg, returns String or usvg::Error)
|
|
--> bullet_svg/lib.rs (Initializes fontdb and parser, calls usvg::Tree to convert svg to usvg)
    |
    |
    |
    --> bullet_wasm/lib.js (javascript/wasm bindings for bullet_svg. Includes js/wasm bindings and wasm_specific memory and error-handling)

```

### Binary Targets
resvg, usvg, bullet_svg, and bullet_tool are included in the workspace and are built in the ./target directory.  Each of these may be built by running ``` cargo build ```

### Webassembly Targets
bullet_wasm is excluded from the workspace and is built using ```wasm-pack build```  The wasm targets are built in bullet_wasm/pkg.  These may be packaged for use with npm, as described below.

#### Webassembly Test
A web assembly test program is in /bullet_wasm/www . Node modules can be installed with ```npm install``` A front-end development server can be launched using ```npm run start``` For deployment, ```npm run build``` creates deployment files in ./bullet_wasm/www/dist that can be copied to a server's static file directory.

#### SVG Arc Support
Shaper has forked resvg to include SVG arcs in the usvg format. Without this, usvg would approximate all arcs with Bezier curves. The feature can be enabled or disabled in the cargo.toml file for bullet_svg. 

Arc support also requires Shaper's fork of the kurbo library (https://github.com/ShaperToolsOSS/kurbo). When using the kurbo fork, you will need to check out the svg_arcs branch locally, and then make sure usvg is compiling with the local version of kurbo instead of the one from crates.io.

### Font Support
usvg accesses fonts stored in a fontdb struct. Fonts may either be bundled statically during compilation or loaded dynamically at run time. See bullet_svg/src/lib.rs for examples of both. The static font files have been omitted to avoid potential licensing issues.

### Units
resvg converts all units to display pixels (96px/in) by default. We have modified usvg to accept arguments for `--dpi_render` and `--dpi_units` dpi_units specifies the pixels per inch to use when converting between units and pixels, and dpi_render specifies the output rendering resolution. 

Additionally, we modified usvg with the function `Tree::to_string_with_unit` to output svg data using any valid svg unit. bullet_svg/lib.rs uses this to convert all incoming svg files to mm output. It also attempts to guess the source of the svg and sets the `dpi_units` value accordingly. This is similar to the functionality on `Tool`.

### NPM Package Export
We have configured the WebAssembly build to export compiled wasm and js binding files as a npm module, allowing it to be imported in any web project. 

To update the npm package, bump the version number in `bullet_wasm/pkg/package.json` and run `npm publish`. You may need to [configure npm to use a personal access token](https://docs.github.com/en/packages/guides/configuring-npm-for-use-with-github-packages#authenticating-with-a-personal-access-token) to access the private packages on Github.

You should also insure that `wasm-pack build` does not overwrite the `pkg/package.json` file. This is an example of a correctly configured package.json file. Note the name value is prefixed with @ShaperTools:
```
{
  "name": "@ShaperTools/bullet_wasm",
  "repository": {
    "type": "git",
    "url": "https://github.com/ShaperToolsOSS/resvg.git"
  },
  "collaborators": [
    "Jon Hollander <jh@shapertools.com>"
  ],
  "description": "WASM bindings for Shaper Tools Bulletproof SVG Parser",
  "version": "0.9.0",
  "license": "UNLICENSED",
  "repository": {
    "type": "git",
    "url": "https://github.com/ShaperToolsOSS/resvg"
  },
  "files": [
    "bullet_wasm_bg.wasm",
    "bullet_wasm_bg.js",
    "bullet_wasm.js",
    "bullet_wasm.d.ts",
    "LICENSE_MIT",
    "LICENSE_APACHE"
  ],
  "module": "bullet_wasm.js",
  "types": "bullet_wasm.d.ts",
  "sideEffects": false
}
```
## resvg
![Build Status](https://github.com/RazrFalcon/resvg/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/resvg.svg)](https://crates.io/crates/resvg)
[![Documentation](https://docs.rs/resvg/badge.svg)](https://docs.rs/resvg)

*resvg* is an [SVG](https://en.wikipedia.org/wiki/Scalable_Vector_Graphics) rendering library.

## Purpose

`resvg` can be used as a Rust library, a C library and as a CLI application
to render SVG files based on a
[static](http://www.w3.org/TR/SVG11/feature#SVG-static)
[SVG Full 1.1](https://www.w3.org/TR/SVG11/) subset.

The core idea is to make a fast, small, portable SVG library designed for edge-cases.
Right now, a `resvg` CLI application is less than 3MiB and doesn't require any external dependencies.

Another major difference from other SVG rendering libraries is that `resvg` does a lot
of preprocessing before rendering. It converts an input SVG into a simplified one
called [Micro SVG](./docs/usvg_spec.adoc) and only then it begins rendering.
So it's very easy to implement a new rendering backend.
But we officially support only one.
And you can also access *Micro SVG* as XML directly via the [usvg](./usvg) tool.

## SVG support

`resvg` is aiming to support only the [static](http://www.w3.org/TR/SVG11/feature#SVG-static)
SVG subset; e.g. no `a`, `script`, `view` or `cursor` elements, no events and no animations.

[SVG Tiny 1.2](https://www.w3.org/TR/SVGTiny12/) and [SVG 2.0](https://www.w3.org/TR/SVG2/)
are not supported and not planned.

Results of the [resvg test suite](./tests/README.md):

![](./.github/chart.svg)

You can find a complete table of supported features
[here](https://razrfalcon.github.io/resvg-test-suite/svg-support-table.html).
It also includes alternative libraries.

## Performance

Comparing performance between different SVG rendering libraries is like comparing
apples and oranges. Everyone has a very different set of supported features,
implementation languages, build flags, etc.
But since `resvg` is written in Rust and uses [tiny-skia] for rendering - it's pretty fast.

## Safety

resvg and most of its dependencies are pretty safe.
The main exceptions are [tiny-skia] and files memory mapping.

## License

`resvg` project is licensed under the [MPLv2.0](https://www.mozilla.org/en-US/MPL/).

[rustybuzz]: https://github.com/RazrFalcon/rustybuzz
[tiny-skia]: https://github.com/RazrFalcon/tiny-skia
