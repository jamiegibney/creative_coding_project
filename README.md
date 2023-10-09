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
- Add methods to change the curve amounts of the `AdsrEnvelope` struct
- Expand the `Ramp` struct with more methods
- Implement EQ curves/nodes w nannou
- Implement polyphony
- Implement distortion
- Implement compression
- Implement a basic reverberator (use the comb filters)
- Implement person noise
- Implement stereo width controls

- Try to plot the frequency response of time-domain filters (try computing the impulse response, then performing an FFT?)
- Try to process the spectrum on separate thread(s) 
- Try using `Arc<Mutex<T>>` instead of `mpsc` channels to interact with the audio thread?
- Try having an "Effect" trait rather than just a "Filter" trait, and than have that implement DynClone. Perhaps it could have a
"`try_clone()`" method for clone attempts? The idea is that every effects processor could support dynamic dispatch (`dyn Effect`),
which might make setting up processing chains a lot easier.
