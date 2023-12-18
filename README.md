# TODO

### Fixes
- Can't properly display pre and post spectrogram meshes simultaneously (earcutr issue?)
- Randomising resonator bank pitches doesn't properly quantise to a scale 
- Fix the spectrogram amplitude scaling
- Fix the spectral filter gain compensation: it differs drastically with different window sizes

### Implementation
- **PRIORITY**: **GUI!**
    - 3-band EQ
    - sliders, knobs, buttons, menus
- Voronoi noise algorithm
    - to control pitch/amp/panning of each resonator
    - to act as a spectral mask
- point distribution/scattering algorithm
- flow field/jitter to add motion to cells
- sequencer for some kind of rhythmic note generation
- DSP:
    - pitch mod effects (chorus, flanger..)
    - compression
    - distortion
        - better waveshaping
        - downsampling
        - frequency shifting
        - ring modulation
        - "hard" clip
    - basic reverberator (use the comb filters?)
- `UIParams` needs to initialise all the audio stuff

### Stuff to try
- try to replace the generative algorithms with compute shaders, and then copy the content from the GPU to the CPU for audio processing 
    - despite the copying overhead, the compute time will be dramatically reduced by the GPU.
- HRTF processing

### Optimisations
- SIMD, *vroom*
    - filters?
    - STFT helper?
    - spectrum processing?
    - oversampling
- Find out why some spectrum points are not finite?
- Oversampling could be much faster - better convolution algorithm? Process both channels in parallel?
