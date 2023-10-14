# TODO

- Get spectrogram working with nannou. See `nannou/examples/draw/draw_mesh.rs` for an example of a mesh being created with nannou.

### Fixes
- **PRIORITY** `AdsrEnvelope` doesn't respond when its attack time is `0.0`
- **PRIORITY** Lowpass biquad filter doesn't seem to process properly in the `AudioModel` struct (`process_filters()`) method
- Convert the spectrogram to nannou from egui
    - Mesh implementation
    - Gradient implementation

### Implementation
- Implement compression
- Distortion
    - Implement downsampling
    - Implement frequency shifting
    - Implement ring modulation
- Implement EQ curves/nodes w nannou
- Implement stereo width controls
- Implement a basic reverberator (use the comb filters?)
- Implement Perlin noise (does nannou have it already?)
- Reimplement musical note representation
- Revise interpolation and transfer functions, as very few of them are tested

### Stuff to try
- Try to plot the frequency response of time-domain filters (try computing the impulse response, then performing an FFT?)
- Try to process the spectrum on separate thread(s) 
- **PRIORITY** Try having an "Effect" trait rather than just a "Filter" trait, and than have that implement DynClone. Perhaps it could have a
"`try_clone()`" method for clone attempts? The idea is that every effects processor could support dynamic dispatch (`dyn Effect`),
which might make setting up processing chains a lot easier.
