# UI Parameters 
A list of all the UI-related parameters for the device.

## Spectral Mask
#### General
- Which masking algorithm is being used
- A button to reset the seed?
- Speed of the mask scan line
- Resolution of the mask
- Pre/post FX

#### Contours
- Number of contours
- Thickness of each contour (should by symmetrical)
- Speed of the animation

#### Smooth Life
- Resolution of the simulation (within reasonable limits?)
- Speed of the animation
- Perhaps a few presets for the internal state parameters (i.e. one for fluid, another for swirly...)

## Spectrograms
- Resolution?
- Timing?
- Ability to show/hide pre/post FX
- Level (to scale the spectrum more appropriately)
    - ***could this be automatic?***

## Resonator Bank
- Current scale and root note
- Number of resonators
- Frequency spread
- Frequency shift
- Frequency inharmonic scale
- Toggle to quantise to scale
- Button to randomise pitches (keyboard shortcut too)

## Post effects

#### Low-pass filter
- Cutoff frequency
- Q

#### High-pass filter
- Cutoff frequency
- Q

#### Ping-pong delay
- Delay time
- Tempo synchronisation
- Feedback
- Mix

#### Distortion
- "Amount" (i.e. drive/frequency)
- Algorithm — only a few options:
    - Smooth soft-clipping
    - Hard xfer function
    - Wrapping xfer function
    - Downsampling

#### Gain/width
- Output gain
- Output stereo width

## EQ
3 bands: low, mid, and high:

#### Low band
- Low-cut/shelf filter toggle
- Cutoff and Q/gain (could be controlled with node)

#### Mid band
- Peak/notch filter toggle
- Cutoff and Q/gain (could be controlled with node)

#### High band
- High-cut/shelf filter toggle
- Cutoff and Q/gain (could be controlled with node)

