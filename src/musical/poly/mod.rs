

pub enum NoteEvent {
    NoteOn { id: Option<i32> },
    NoteOff { note: u8, id: Option<i32> },
}
