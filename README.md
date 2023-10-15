# TODO

- Get spectrogram working with nannou. See `nannou/examples/draw/draw_mesh.rs` for an example of a mesh being created with nannou.

### Fixes
- **PRIORITY** `AdsrEnvelope` doesn't respond when its attack time is `0.0`
- Can't properly display pre and post spectrogram meshes simultaneously (earcutr issue?)

### Implementation
- Implement compression
- Implement other allpass filter design, and look into the diopser implementation
- Distortion
    - Implement downsampling
    - Implement frequency shifting
    - Implement ring modulation
- Implement EQ curves/nodes w nannou
- Implement stereo width controls
- Implement a basic reverberator (use the comb filters?)
- Implement Perlin noise (does nannou have it already?)
- Revise interpolation and transfer functions, as very few of them are tested

### Stuff to try
- Try to plot the frequency response of time-domain filters (try computing the impulse response, then performing an FFT?)
- Try to process the spectrum on the GPU or another thread
- **PRIORITY** Try having an "Effect" trait rather than just a "Filter" trait, and than have that implement DynClone. Perhaps it could have a "`try_clone()`" method for clone attempts? The idea is that every effects processor could support dynamic dispatch (`dyn Effect`), which might make setting up processing chains a lot easier.
- HRTF processing

### Optimisations
- SIMD, *vroom*
    - filters?
    - STFT helper?
    - spectrum processing?
- Point decimation could mutate a buffer in-place
- Interpolating between frequency bins involves a lot of recomputation, basic caching could speed up quite a bit
- Audio thread processes spectrum analysers - it may be better to simply copy the buffer pre- and post-fx, and then use that data to process the spectral data later (or on another thread? could a thread pool work?)
- Find out why some spectrum points are not finite?

