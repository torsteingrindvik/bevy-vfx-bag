# Bevy Visual Effects Bag

This crate has an assortment of effects easily applied to Bevy apps via plugins.

Here is a showcase made by adding some visual effects to Bevy's breakout example.
Note that the game itself is not modified except visually.

[Breakout Video](https://user-images.githubusercontent.com/52322338/198832540-a1a2fa00-8c75-4f55-a3e7-b4ce18fef958.mp4)

The effects added in the above are:

- A raindrop effect (due to effects applied after this one it gives the impression of static noise across the screen).
- Chromatic aberration.
- Waves. This effect causes the general "waviness" seen as well as the collision shake effect.
- A CRT mask. This changes the edges of the screen to resemeble a CRT.
- Pixelation. This pixelates the image. The demo increases this effect slightly each time a collision happens, but it's a bit hard to notice due to the short length of the video.
- Color grading. A color lookup texture is applied, which changes the color output accordingly.

Scroll down to see videos of the examples in contained in this repo.

## Bevy compatibility

When adding this crate as a dependency in your project,
the Bevy version you use will need to match up according to the
following table:

|bevy-vfx-bag|bevy|
|---|---|
|0.1.0|0.9.0|

## Getting started

The general strategy is:

* Add the main plugin: `BevyVfxBagPlugin`.
* Add the effect plugin you are interested in.
* Add the `PostProcessingInput` marker component to your camera. This camera's output is then used as input for the post processing effects.
* Add any systems to change effect parameters at runtime.

```rust,ignore
// See the examples folder for fleshed out examples,
// this just shows the general strategy.

fn main(){
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(BevyVfxBagPlugin) // This needs to be added for any effect to work
    .add_plugin(FlipPlugin) // This needs to be added for the flip effect to work
    .add_startup_system(setup)
    .add_system(update)
    .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle { ... })
        .insert(PostProcessingInput); // Marking the camera is important!
}

fn update(mut flip: ResMut<Flip>) {
    // Here I can change the parameters of this effect at runtime.
}
```

## Limitations

- You can only use a single camera as the source for effects.
- You cannot change the order of applied effects at runtime- this is decided by plugin insertion order when making the `App`.
- You can toggle effects off/on at runtime- but the shaders will still run. They simply pass through the input image to the output, but this requires a texture sample. Therefore there is likely a slight performance cost for added-but-disabled effects.

## Examples

All videos below are captured from running the [examples](https://github.com/torsteingrindvik/bevy-vfx-bag/tree/main/examples).

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

### LUT

Allows color grading via look-up textures.
There is also an example to generate the neutral LUT, `cargo r --example make-neutral-lut`.
This file can then be modified in any image editor in order to replicate the look/feeling you're after.

The plugin allows splitting the image vertically (shown in the video), which can be used to compare the look
before and after color grading.

[LUT Example Video](https://user-images.githubusercontent.com/52322338/196005149-a76e6d5b-d227-4e71-9f3f-4e1d86b4d12e.mp4)

### Pixelate

Pixelate the screen.
The user controls how big the block size of a "pixel" is.

[Pixelate Example Video](https://user-images.githubusercontent.com/52322338/197244451-017846fa-789d-49a7-a622-bc70d5397b4c.mp4)

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

### T-Rex

Shows another use of the wave effect.
By having a violent wave effect we can simulate something like the earth shaking due to something
big approaching by toggling the effect in intervals.

[T-Rex Example Video](https://user-images.githubusercontent.com/52322338/198832244-4898bbe9-4b24-4ddb-a5d5-665f3ecc71e3.mp4)

### Underwater

Underwater (ish) effect.
Shoehorns most effects into one example, to show how they may be stacked.

[Underwater Example Video](https://user-images.githubusercontent.com/52322338/196513586-dbf0f822-0144-4af8-b11d-177fada21311.mp4)

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
