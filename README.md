# TODO

- Get spectrogram working with nannou. See `nannou/examples/draw/draw_mesh.rs` for
an example of a mesh being created in nannou.

- Fix envelope values not responding to `0.0`
- Fix glissando issue (ramp problem it seems)
- Fix ring buffer (probably best to re-implement from scratch)

- Convert the spectrogram to nannou from egui, which will require:
    - Mesh implementation
    - Gradient implementation
- Add filters in the comb filter feedback loop (maybe have a callback in the comb filter?)
- Expand the `Ramp` struct with more methods
- Add methods to change the curve amounts of the `AdsrEnvelope` struct
- Implement EQ curves/nodes w nannou
- Try to plot the frequency response of time-domain filters (try computing the impulse response, then performing an FFT?)
- Implement polyphony
- Implement distortion
- Implement compression
- Implement a basic reverberator (use the comb filters)
- Implement person noise
- Implement stereo width controls
- Try to process the spectrum on separate thread(s) 
