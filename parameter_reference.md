# Parameter reference

## Resonator Field

#### Scale
- **`Root note`** (default `C`): the root note of the resonator bank's current scale.
- **`Scale`** (default `Maj Pent.`): the scale to use for the resonator bank..
- **`Quantise`** (default `Quantise On`): whether each resonator pitch should be quantised to the set scale.

#### Resonator settings
- **`Frequency shift`** (default `0.0 st`): the amount each resonator's pitch is shifted. Applied before quantisation.
- **`Frequency spread`** (default `0.5`): the total range of which resonator pitches can be distributed. A value of `1.0` corresponds to 9 octaves of range, and a value of `0.0` corresponds to no variation. Applied before quantisation.
- **`Inharmonic`** (default `0.3`): how much quantised resonator pitches skew towards their original un-quantised values.
- **`Panning`** (default `1.0`): how much the horizontal position of each node affects the panning of its attached resonator.

#### Field settings
- **`Resonators`** (default `8`): how many resonators are active at a time.
- **`Friction`** (default: `0.5`): how resistance each node is to motion from the `Regenerate` and `Push` controls.
- **`Mix`** (default `100 %`): the dry/wet mix of the resonator bank.
- **`Exciter`** (default: `Noise`): the type of oscillator to use.

#### Buttons
- **`Regenerate`**: randomises the position of each of the resonator nodes.
- **`Push`**: applies a small force to each of the resonator nodes.

## Spectral Filter

#### Filter settings
- **`Mix`** (default `100 %`): the dry/wet mix of the spectral filter.
- **`Resolution`** (default `1024`): the block size of the spectral filter. Smaller sizes increase time resolution (i.e., how "fast" the filter responds), but reduce frequency resolution. Larger sizes will increase latency.
- **`Scan line speed`** (default `1.0`): the speed of the scan line.
- **`Algorithm`** (default: `Contours`): the visual algorithm to use for the spectral filter mask.

#### Contours
Contour lines of a Perlin noise field.
- **`Speed`** (default `0.2`): the speed at which the noise algorithm moves.
- **`Thickness`** (default `0.6`): the thickness of each contour line.
- **`Count`** (default `8`): how many contour lines to draw (not overall).

#### SmoothLife
Botched SmoothLife simulation.
- **`Preset`** (default `Jitter`): the SmoothLife parameters to use. Available presets are:
    - `Jitter`: distorted, jittering waves which move diagonally.
    - `Slime`: thick, smearing diagonal lines.
    - `Corrupt`: flickering, unstable digital patterns. Your GPU power affects the appearance of this preset.

#### Voronoi
Voronoi cells derived from moving points.
- **`Speed`** (default `0.3`): the speed at which the cells move about.
- **`Weight`** (default `0.65`): the weight of the cell borders and "isolines".
- **`Count`** (default `10`): the total number of cells.

## Parametric EQ/Spectrogram
#### Spectrogram
- **`View`** (default `Pre/Post`): which spectrogram stages to draw.


#### Low Filter
- **`Type`** (default `Cut`): the filter type to use (high cut or low shelf).
- **`Cutoff`** (default `500 Hz`): the filter cutoff frequency.
- **`Q`** (`Cut` only, default `0.707`): the cut filter's Q value.
- **`Gain`** (`Shelf` only, default `0.0 dB`): the shelf filter's gain value.

#### Low Filter
- **`Type`** (default `Cut`): the filter type to use (low cut or high shelf).
- **`Cutoff`** (default `500 Hz`): the filter cutoff frequency.
- **`Q`** (`Cut` only, default `0.707`): the cut filter's Q value.
- **`Gain`** (`Shelf` only, default `0.0 dB`): the shelf filter's gain value.

## Effects
- **`Pre-FX Gain`** (default: `0.0 dB`): the amount of gain to apply pre-FX.

#### Distortion
- **`Amount`** (default: `0.0`): the amount of distortion to apply. Effect differs per distortion algorithm.
- **`Type`** (default: `Type`): distortion algorithm to apply.

#### Delay
- **`Time`** (default: `250 ms`): time between delay taps.
- **`Feedback`** (default: `75 %`): amount of delay feedback.
- **`Ping-pong`** (default: `On`): whether to cross-feed channels to create a ping-pong effect.
- **`Mix`** (default: `0 %`): the dry/wet mix of the delay.

#### Compression
- **`Threshold`** (default: `-12 dB`): the compressor's threshold in decibels.
- **`Ratio`** (default: `10:1`): the compressor's ratio.
- **`Attack`** (default: `80 ms`): compression attack time in milliseconds.
- **`Release`** (default: `200 ms`): compression release time in milliseconds.
