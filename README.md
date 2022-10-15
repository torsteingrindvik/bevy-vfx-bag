# Bevy Visual Effects Bag

This crate has an assortment of effects easily applied to Bevy apps via plugins.

## Examples

All videos are captured from running the examples.

Do `cargo r --example` in this repository to get a list of examples you may run.
Some examples use keyboard/mouse input to change parameters at runtime as well.

### Blur

Shows blurring an image.

The strength of the blur is changed, as well as the radius.

The radius refers to far away texels are sampled relative to the origin texel.

[Blur Example Video](https://user-images.githubusercontent.com/52322338/195917033-762688ae-c8ce-4d62-9446-900cd6af1939.mp4)

### Chromatic Aberration

Shows chromatic aberration.
The red, green, and blue channels are offset from the original locations in the image.

The direction of these offsets as well as their magnitudes are controllable.
The example has the directions animated over time at different speeds.
The user controls the magnitudes.

[Chromatic Aberration Example Video](https://user-images.githubusercontent.com/52322338/195917082-453ea4e7-d7b8-46c3-ad6d-1298e53620c0.mp4)

### Flip

Allows flipping the input image horizontally, vertically, or both.

[Flip Example Video](https://user-images.githubusercontent.com/52322338/195917100-acece75a-a867-43c8-a850-62ca7a1109f0.mp4)

### Raindrops

Shows raindrops on the screen.
The users controls zooming of the raindrops, which affects how large they appear on screen.

The intensity determines how much a raindrop will distort sampling the original image.
This in effect is "how much light bends" through the drop.

Some drops are animated. The speed of this repeating animation is controlled too.

[Raindrops Example Video](https://user-images.githubusercontent.com/52322338/195917577-352f549b-1622-4e62-b2e9-7005fbbdd875.mp4)

### Vignette

Shows the vignette effect.

The example shows changing the "feathering" of the effect.
This means how large the smooth transition zone between original image and vignette is.

Not shown is changing the radius, and changing the color of the vignette.

[Vignette Example Video](https://user-images.githubusercontent.com/52322338/195917174-0be12446-d527-4d81-8e0d-24370b8bdd03.mp4)

### Wave

Shows displacing original image sampling in waves over the screen.

The number of waves, how fast they travel (they are sinusoidal),
and their amplitudes are controllable separately for the horizontal and
vertical axes.

This is quite flexible and can create interesting effects.

A possibility not shown in the video is a camera shake effect,
which might be achieved by having a high number of waves at high speed with low amplitude,
and quickly dampening those parameters to zero so the effect ends.

[Wave Example Video](https://user-images.githubusercontent.com/52322338/195917192-461fd2a1-8bdf-4671-bfce-a1182de41fb1.mp4)

