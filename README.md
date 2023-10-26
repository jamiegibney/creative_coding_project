# TODO

### Fixes
- The DSP idle detection is too quick for notes with very quick release times for the spectrograms. Try to add a timer after all voices are finished which leaves enough time for the spectrogram to decay properly
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
- Implement a version of the poisson distribution, which may help for some randomised sequencing/note events
- Implement FFT averaging for the spectrograms, which might solve the noisy high frequency information

### Stuff to try
- Try to plot the frequency response of time-domain filters (try computing the impulse response, then performing an FFT?)
- Try to utilise the GPU?
- **PRIORITY** Try having an "Effect" trait rather than just a "Filter" trait, and than have that implement DynClone. Perhaps it could have a "`try_clone()`" method for clone attempts? The idea is that every effects processor could support dynamic dispatch (`dyn Effect`), which might make setting up processing chains a lot easier
- HRTF processing

### Optimisations
- SIMD, *vroom*
    - filters?
    - STFT helper?
    - spectrum processing?
- Point decimation could mutate a buffer in-place
- Interpolating between frequency bins involves a lot of recomputation, basic caching could speed up quite a bit
- Find out why some spectrum points are not finite?
