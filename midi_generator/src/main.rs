extern crate rand;
extern crate rand_distr;

use rand::Rng;
use rand::distributions::WeightedIndex;
use rand_distr::{Distribution, Uniform};

#[derive(Debug, Copy, Clone)]
/// Enum defining all MIDIEvents
/// 
/// Used with match to create different events
/// Use MIDIEvent::pick_random() to randomly choose a MIDIEvent with uniform distribution
enum MIDIEvent {
    NoteOff,
    NoteOn,
    PolyphonicPressure,
    Controller,
    ProgramChange,
    ChannelPressure,
    PitchBend,
}

impl MIDIEvent {
    /// Returns a random MDIIEvent using a Uniform distribution
    fn pick_random() -> MIDIEvent {
        let mut rng = rand::thread_rng();
        let temp = Uniform::from(0..7).sample(&mut rng) as u32;
        match temp {
            0 => MIDIEvent::NoteOff,
            1 => MIDIEvent::NoteOn,
            2 => MIDIEvent::PolyphonicPressure,
            3 => MIDIEvent::Controller,
            4 => MIDIEvent::ProgramChange,
            5 => MIDIEvent::ChannelPressure,
            6 => MIDIEvent::PitchBend,
            _ => panic!("Error when picking random MIDIEvent. Number out of bounds.")
        }
    }
}

#[derive(Debug, Copy, Clone)]
/// Enum defining all MetaEvents
/// 
/// Used with match to create different events
/// Use MetaEvent::pick_random() to randomly choose a MetaEvent with uniform distribution
enum MetaEvent {
    Text,
    SequenceORTrackName,
    InstrumentName,
    Lyric,
    ProgramName,
    MIDIChannelPrefix,
    MIDIPort,
    EndOfTrack,
    Marker,
    CuePoint,
    Tempo,
    TimeSignature,
    KeySignature,
}

impl MetaEvent {

    /// Returns a random MetaEvent using a Uniform distribution
    /// 
    /// # Arguments
    /// 
    /// * `lower` - A u32 representing the lower bound of the random number generation, minimum value of 0
    /// * `upper` - A u32 representing the upper bound of the random number generation, maximum value of 13
    /// 
    /// To pick between timing events, Lower: 8 and Upper: 13
    fn pick_random(lower: u32, upper: u32) -> MetaEvent {
        let mut rng = rand::thread_rng();
        let temp = Uniform::from(lower..upper).sample(&mut rng) as u32;
        match temp {
            0 => MetaEvent::Text,
            1 => MetaEvent::SequenceORTrackName,
            2 => MetaEvent::InstrumentName,
            3 => MetaEvent::Lyric,
            4 => MetaEvent::ProgramName,
            5 => MetaEvent::MIDIChannelPrefix,
            6 => MetaEvent::MIDIPort,
            7 => MetaEvent::EndOfTrack,
            8 => MetaEvent::Marker,
            9 => MetaEvent::CuePoint,
            10 => MetaEvent::Tempo,
            11 => MetaEvent::TimeSignature,
            12 => MetaEvent::KeySignature,
            _ => panic!("Error when picking random MetaEvent. Number out of bounds.")
        }
    }
}

#[derive(Debug)]
struct MThd {
    //identifier: String,
    identifier: [u8; 4],
    chunklen: u32, // big-endian
    format: u16, // big-endian
    ntracks: u16, // big-endian
    tickdiv: u16, // big-endian
}

impl MThd {

    /// Create a new MThd chunk to serve as the header of the MIDI file
    /// 
    /// Randomly choosese format, ntracks, and tickdiv with uniform distribution and common values
    fn new() -> MThd {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::from(0..3);

        let fmt = uniform.sample(&mut rng) as u16;
        
        let ntrk = match fmt {
            0 => 1,// format 0 can only contain 1 MTrk chunk
            1 => Uniform::from(2..26).sample(&mut rng) as u16, // 2 or more MTrk chunks, played simultaneously, let's set an arbitrary limit of 25
            2 => Uniform::from(1..26).sample(&mut rng) as u16, // 1 or more MTrk chunks, played independently
            _ => panic!("Error found when generating MThd chunk. Invalid ntracks")
        };

        /*
        tickdiv : specifies the timing interval to be used, and whether timecode (Hrs.Mins.Secs.Frames) or metrical (Bar.Beat) timing is to be used. 
        With metrical timing, the timing interval is tempo related, whereas with timecode the timing interval is in absolute time, and hence not related to tempo.

        Bit 15 (the top bit of the first byte) is a flag indicating the timing scheme in use :
        
        Bit 15 = 0 : metrical timing
        Bits 0 - 14 are a 15-bit number indicating the number of sub-divisions of a quarter note (aka pulses per quarter note, ppqn). 
        A common value is 96, which would be represented in hex as 00 60. You will notice that 96 is a nice number for dividing by 2 or 3 (with further repeated halving), 
        so using this value for tickdiv allows triplets and dotted notes right down to hemi-demi-semiquavers to be represented.
        
        Bit 15 = 1 : timecode
        Bits 8 - 15 (i.e. the first byte) specifies the number of frames per second (fps), and will be one of the four SMPTE standards - 24, 25, 29 or 30, 
        though expressed as a negative value (using 2's complement notation), as follows :
        
        fps	Representation (hex)
        24  E8
        25  E7
        29  E3
        30	E2

        Bits 0 - 7 (the second byte) specifies the sub-frame resolution, i.e. the number of sub-divisions of a frame. Typical values are 4 (corresponding to MIDI Time Code), 8, 10, 80 (corresponding to SMPTE bit resolution), or 100.
        A timing resolution of 1 ms can be achieved by specifying 25 fps and 40 sub-frames, which would be encoded in hex as  E7 28.
        */

        let timecode = Uniform::from(0..2).sample(&mut rng) as u16; // get a 0 or 1 for bit 15
        let mut tckdv: u16 = timecode << 15;

        let tckdv_extra_bits: u16 = match timecode {
            0 => 96, // common value
            1 => {
                let mut temp: u16 = match Uniform::from(0..4).sample(&mut rng) as u8 { // this gets us our fps
                    0 => 0xE8 as u16,
                    1 => 0xE7 as u16,
                    2 => 0xE3 as u16,
                    3 => 0xE2 as u16,
                    _ => panic!("Error found when generating MThd chunk. Invalid fps in tickdiv.")
                };
                temp = temp << 8; /* set up bits 8 - 15 and shift */
                // temp = temp | (1 << 15); /* because we had to move bit 0 over by 8, bit 7 may have overwritten bit 15 with a 0, let's do this for safety */
                temp = temp | match Uniform::from(0..5).sample(&mut rng) as u8 { /* set up our sub-frame resolution using the typical values */
                    0 => 4 as u16,
                    1 => 8 as u16,
                    2 => 10 as u16,
                    3 => 80 as u16,
                    4 => 100 as u16,
                    _ => panic!("Error found when generating MThd chunk. Invalid sub-frame resolution in tickdiv.")
                };
                temp
            },
            _ => panic!("Error found when generating MThd chunk. Invalid timecode in tickdiv.")
        };

        tckdv = tckdv | tckdv_extra_bits;

        MThd {
            identifier: ['M' as u8, 'T' as u8, 'h' as u8,'d' as u8],
            chunklen: 6, // MIDI currently only supports chunklen 6
            format: fmt,
            ntracks: ntrk,
            tickdiv: tckdv,
        }
    }
}

#[derive(Debug)]
/// This is just a wrapper around a Vec<u8>
struct DeltaTime {
    data: Vec<u8>,
}

fn create_delta_time() -> DeltaTime {
    let mut delta_time = Vec::new();

    let mut rng = rand::thread_rng();

    let choices = [1, 2, 3, 4];
    let weights = [80, 12, 6, 2];
    let dist = WeightedIndex::new(&weights).unwrap();

    let nbytes = choices[dist.sample(&mut rng)];

    // loosely generating weights to ensure that fewer bytes are more common
    // let nbytes: u8 = match Uniform::from(0..20).sample(&mut rng) as u8 {
    //     0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 => 1,
    //     14 | 15 | 16 => 2,
    //     17 | 18 => 3,
    //     19 => 4,
    //     _ => panic!("Error when generating delta time."),
    // };
    
    match nbytes {
        1 => {
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8);
        },
        2 => {
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8 | 0x80);
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8);
        },
        3 => {
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8 | 0x80);
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8 | 0x80);
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8);

        },
        4 => {
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8 | 0x80);
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8 | 0x80);
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8 | 0x80);
            delta_time.push(Uniform::from(0..128).sample(&mut rng) as u8);
        },
        _ => panic!("Error when generating delta time. nbytes out of range.")
    }

    DeltaTime {
        data: delta_time,
    }
}

#[derive(Debug)]
/// This is just a wrapper around a Vec<u8>
struct Event {
    data: Vec<u8>, // Events have variable sizes, so we'll make a vector of bytes
}

impl Event {

    fn new_midi_event(event: MIDIEvent) -> Event {
        let mut event_bytes: Vec<u8> = Vec::new();
        
        let mut rng = rand::thread_rng();

        match event {
            MIDIEvent::NoteOff => {
                let mut status_byte: u8 = 0x80;
                status_byte = status_byte | (Uniform::from(0..16).sample(&mut rng) as u8);
                event_bytes.push(status_byte);

                let note: u8 = Uniform::from(0..128).sample(&mut rng) as u8;
                let velocity: u8 = Uniform::from(0..128).sample(&mut rng) as u8; // defaults to 64 in absence of velocity sensors?
                event_bytes.push(note);
                event_bytes.push(velocity);
            },
            MIDIEvent::NoteOn => {
                let mut status_byte: u8 = 0x90;
                status_byte = status_byte | (Uniform::from(0..16).sample(&mut rng) as u8);
                event_bytes.push(status_byte);

                let note: u8 = Uniform::from(0..128).sample(&mut rng) as u8;
                let velocity: u8 = Uniform::from(0..128).sample(&mut rng) as u8; // defaults to 64 in absence of velocity sensors?
                event_bytes.push(note);
                event_bytes.push(velocity);
            },
            MIDIEvent::PolyphonicPressure => {
                let mut status_byte: u8 = 0xA0;
                status_byte = status_byte | (Uniform::from(0..16).sample(&mut rng) as u8);
                event_bytes.push(status_byte);

                let note: u8 = Uniform::from(0..128).sample(&mut rng) as u8;
                let pressure: u8 = Uniform::from(0..128).sample(&mut rng) as u8; // amount of note aftertouch
                event_bytes.push(note);
                event_bytes.push(pressure);
            },
            MIDIEvent::Controller => {
                let mut status_byte: u8 = 0xB0;
                status_byte = status_byte | (Uniform::from(0..16).sample(&mut rng) as u8);
                event_bytes.push(status_byte);

                let controller: u8 = Uniform::from(0..128).sample(&mut rng) as u8;
                let value: u8 = Uniform::from(0..128).sample(&mut rng) as u8;
                event_bytes.push(controller);
                event_bytes.push(value);
            },
            MIDIEvent::ProgramChange => {
                let mut status_byte: u8 = 0xC0;
                status_byte = status_byte | (Uniform::from(0..16).sample(&mut rng) as u8);
                event_bytes.push(status_byte);

                let program: u8 = Uniform::from(0..128).sample(&mut rng) as u8;
                event_bytes.push(program);
            },
            MIDIEvent::ChannelPressure => {
                let mut status_byte: u8 = 0xD0;
                status_byte = status_byte | (Uniform::from(0..16).sample(&mut rng) as u8);
                event_bytes.push(status_byte);

                let pressure: u8 = Uniform::from(0..128).sample(&mut rng) as u8;
                event_bytes.push(pressure);
            },
            MIDIEvent::PitchBend => {
                let mut status_byte: u8 = 0xE0;
                status_byte = status_byte | (Uniform::from(0..16).sample(&mut rng) as u8);
                event_bytes.push(status_byte);

                let lsb: u8 = Uniform::from(0..128).sample(&mut rng) as u8;
                let msb: u8 = Uniform::from(0..128).sample(&mut rng) as u8;
                event_bytes.push(lsb);
                event_bytes.push(msb);
            },
        }

        Event {
            data: event_bytes,
        }
    }

    fn new_meta_event(event: MetaEvent) -> Event {
        
        let mut event_bytes: Vec<u8> = Vec::new();
        event_bytes.push(0xFF); // Status byte 0xFF holds for all Meta Events

        let mut rng = rand::thread_rng();

        match event {
            MetaEvent::Text => {
                event_bytes.push(0x01);
                let length = Uniform::from(1..50).sample(&mut rng) as u8;
                event_bytes.push(length);
                for byte in generate_random_characters(length as u32) {
                    event_bytes.push(byte);
                }
            },
            MetaEvent::SequenceORTrackName => { // Optional, if in first track of format 0 or 1, gives Sequence Name. Gives Track Name otherwise.
                event_bytes.push(0x03);
                let length = Uniform::from(1..50).sample(&mut rng) as u8;
                event_bytes.push(length);
                for byte in generate_random_characters(length as u32) {
                    event_bytes.push(byte);
                }
            },
            MetaEvent::InstrumentName => {
                event_bytes.push(0x04);
                let length = Uniform::from(1..50).sample(&mut rng) as u8;
                event_bytes.push(length);
                for byte in generate_random_characters(length as u32) {
                    event_bytes.push(byte);
                }
            },
            MetaEvent::Lyric => {
                event_bytes.push(0x05);
                let length = Uniform::from(1..50).sample(&mut rng) as u8;
                event_bytes.push(length);
                for byte in generate_random_characters(length as u32) {
                    event_bytes.push(byte);
                }
            },
            MetaEvent::ProgramName => {
                event_bytes.push(0x08);
                let length = Uniform::from(1..50).sample(&mut rng) as u8;
                event_bytes.push(length);
                for byte in generate_random_characters(length as u32) {
                    event_bytes.push(byte);
                }
            },
            MetaEvent::MIDIChannelPrefix => {
                event_bytes.push(0x20);
                event_bytes.push(0x01);
                event_bytes.push(Uniform::from(0..16).sample(&mut rng) as u8); // cc byte, specifying MIDI channel 0-15
            },
            MetaEvent::MIDIPort => {
                event_bytes.push(0x21);
                event_bytes.push(0x01);
                event_bytes.push(Uniform::from(0..128).sample(&mut rng) as u8); // pp byte, specifying MIDI port 0-127
            },
            MetaEvent::EndOfTrack => { // Mandatory as the last event in each MTrk chunk, only one occurrence per track
                event_bytes.push(0x2F);
                event_bytes.push(0x00); // Length of 0
            },
            MetaEvent::Marker => { // Format 1, only in first MTrk chunk
                event_bytes.push(0x06);
                let length = Uniform::from(1..50).sample(&mut rng) as u8;
                event_bytes.push(length);
                for byte in generate_random_characters(length as u32) {
                    event_bytes.push(byte);
                }
            },
            MetaEvent::CuePoint => { // Format 1, only in first MTrk chunk
                event_bytes.push(0x07);
                let length = Uniform::from(1..50).sample(&mut rng) as u8;
                event_bytes.push(length);
                for byte in generate_random_characters(length as u32) {
                    event_bytes.push(byte);
                }
            },
            MetaEvent::Tempo => { // Format 1, only in first MTrk chunk, Mandatory
                event_bytes.push(0x51);
                event_bytes.push(0x03);

                // Need a 24-bit value for number of microseconds per quarter note
                // set an arbitrary range from 100000..5000000
                let tt_bytes = Uniform::from(100_000..5_000_000).sample(&mut rng) as u32;
                
                event_bytes.push(((tt_bytes & 0xFF0000) >> 16) as u8);
                event_bytes.push(((tt_bytes & 0x00FF00) >> 8) as u8);
                event_bytes.push((tt_bytes & 0x0000FF) as u8);
            },
            MetaEvent::TimeSignature => { // Format 1, only in first MTrk chunk, mandatory
                event_bytes.push(0x58);
                event_bytes.push(0x04);

                // nn byte specifies the numerator of the time signature
                let nn: u8 = Uniform::from(1..33).sample(&mut rng) as u8;
                // dd byte specifies the denominator of the time signature as a negative power of 2 (i.e., 2 is quarter note, 3 is eighth-note, etc.)
                let dd: u8 = Uniform::from(0..7).sample(&mut rng) as u8;
                // cc byte specifies the number of MIDI clocks between metronome clicks
                let cc: u8 = Uniform::from(1..65).sample(&mut rng) as u8;
                // bb byte specifies the number of notated 32nd notes in a MIDI quarter-note (24 MIDI Clocks). The usual value is 8, though some sequencers allow user to specify
                let bb: u8 = 0x08 as u8;

                event_bytes.push(nn);
                event_bytes.push(dd);
                event_bytes.push(cc);
                event_bytes.push(bb);
            },
            MetaEvent::KeySignature => { // Format 1, only in first MTrk chunk, mandatory
                event_bytes.push(0x59);
                event_bytes.push(0x02);

                // sf byte specifies the number of flats or sharps in the key signature, possible values from -7 to +7, inclusive
                let sf: i8 = Uniform::from(-7..8).sample(&mut rng) as i8;
                // mi byte specifies major (0) or minor (1) key
                let mi: u8 = Uniform::from(0..2).sample(&mut rng) as u8;

                event_bytes.push(sf as u8); // cast to u8 will distort the value if we print it, but the bytes are the same
                event_bytes.push(mi);
            },
        }

        Event {
            data: event_bytes,
        }
    }

    fn generate_mandatory_meta_events() -> Vec<Event> {
        
        let mut events: Vec<Event> = Vec::new();

        let mut rng = rand::thread_rng();

        let mut tempo_bytes: Vec<u8> = Vec::new();
        let mut time_signature_bytes: Vec<u8> = Vec::new();
        let mut key_signature_bytes: Vec<u8> = Vec::new();
        
        // Generate a Tempo event

        tempo_bytes.push(0xFF);
        tempo_bytes.push(0x51);
        tempo_bytes.push(0x03);

        // Need a 24-bit value for number of microseconds per quarter note
        // set an arbitrary range from 100000..5000000
        let tt_bytes = Uniform::from(100_000..5_000_000).sample(&mut rng) as u32;
        
        tempo_bytes.push(((tt_bytes & 0xFF0000) >> 16) as u8);
        tempo_bytes.push(((tt_bytes & 0x00FF00) >> 8) as u8);
        tempo_bytes.push((tt_bytes & 0x0000FF) as u8);
            

        let tempo = Event {
            data: tempo_bytes,
        };

        // Generate a Time Signature event

        time_signature_bytes.push(0xFF);
        time_signature_bytes.push(0x58);
        time_signature_bytes.push(0x04);

        // nn byte specifies the numerator of the time signature
        let nn: u8 = Uniform::from(1..33).sample(&mut rng) as u8;
        // dd byte specifies the denominator of the time signature as a negative power of 2 (i.e., 2 is quarter note, 3 is eighth-note, etc.)
        let dd: u8 = Uniform::from(0..7).sample(&mut rng) as u8;
        // cc byte specifies the number of MIDI clocks between metronome clicks
        let cc: u8 = Uniform::from(1..65).sample(&mut rng) as u8;
        // bb byte specifies the number of notated 32nd notes in a MIDI quarter-note (24 MIDI Clocks). The usual value is 8, though some sequencers allow user to specify
        let bb: u8 = 0x08 as u8;

        time_signature_bytes.push(nn);
        time_signature_bytes.push(dd);
        time_signature_bytes.push(cc);
        time_signature_bytes.push(bb);
            
        let time_signature = Event {
            data: time_signature_bytes,
        };

        // Generate a Key Signature event

        key_signature_bytes.push(0xFF);
        key_signature_bytes.push(0x59);
        key_signature_bytes.push(0x02);

        // sf byte specifies the number of flats or sharps in the key signature, possible values from -7 to +7, inclusive
        let sf: i8 = Uniform::from(-7..8).sample(&mut rng) as i8;
        // mi byte specifies major (0) or minor (1) key
        let mi: u8 = Uniform::from(0..2).sample(&mut rng) as u8;

        key_signature_bytes.push(sf as u8); // cast to u8 will distort the value if we print it, but the bytes are the same
        key_signature_bytes.push(mi);

        let key_signature = Event {
            data: key_signature_bytes,
        };

        // Add events to vector

        events.push(tempo);
        events.push(time_signature);
        events.push(key_signature);

        events
    }
}

// Track chunk
// A single track chunk will contain a sequence of delta-time / event pairs for chunklen bytes
// The different event types, MidiEvent, SysExEvent, and MetaEvent can all be used in a single track chunk
#[derive(Debug)]
struct MTrk {
    identifier: [u8; 4],
    chunklen: u32, // big-endian
    //data: Vec<(DeltaTime, Event)>,
    data: Vec<(DeltaTime, Event)>,
}

impl MTrk {
    fn new() -> MTrk {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::from(0..3);

        let fmt = uniform.sample(&mut rng) as u16;

        MTrk {
            identifier: ['M' as u8, 'T' as u8, 'r' as u8, 'k' as u8],
            chunklen: 3,
            data: Vec::new(),
        }
    }

    fn new_track_format_0() -> MTrk {
        todo!();
    }

    /// Generates a random Global Tempo Track Chunk for use in format 1 files.
    /// A global tempo track contains all timing related events and no note data.
    /// 
    /// This will generate a random number of timing events from 1..100
    /// 
    /// Timing events are the following Meta events:
    /// 
    /// * Marker
    /// * Cue Point
    /// * Tempo
    /// * SMPTE Offset
    /// * Time Signature
    /// * Key Signature
    fn new_global_tempo() -> MTrk {
        let mut rng = rand::thread_rng();
        
        // Generate <DeltaTime, Event> pairs
        

        MTrk {
            identifier: ['M' as u8, 'T' as u8, 'r' as u8, 'k' as u8],
            chunklen: 3,
            data: Vec::new(),
        }

    }

    fn new_track_format_1() -> MTrk {
        todo!();
    }

    fn new_track_format_2() -> MTrk {
        todo!();
    }
}

/// Generate n number of ASCII characters in range 32..127 (inclusive), returning as a Vec<u8>
/// 
/// # Arguments
/// 
/// * `n` - The number of characters to generate
fn generate_random_characters(n: u32) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let uniform = Uniform::from(32..128);

    let mut chars = Vec::new();

    for _ in 0..n {
        chars.push(uniform.sample(&mut rng) as u8);
    }

    chars
}

fn main() {
    let header = MThd::new();
    let mut tracks = Vec::new();

    // Generate MTrk chunks depending on format
    if header.format == 0 { // need a single MTrk chunk containing any valid event
        tracks.push(MTrk::new_track_format_0());
    }
    else if header.format == 1 { // first MTrk chunk is a global tempo chunk, second and subsequent are the actual note data
        tracks.push(MTrk::new_global_tempo());
        for _ in 1..header.ntracks {
            tracks.push(MTrk::new_track_format_1());
        }        
    } 
    else { // each track is separate and can contain any type of event, each track may have its own tempo map
        for _ in 0..header.ntracks {
            tracks.push(MTrk::new_track_format_2());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mthd_size_is_valid() {
        let header = MThd::new();
        assert_eq!(
            std::mem::size_of_val(&header.identifier) +
            std::mem::size_of_val(&header.chunklen) +
            std::mem::size_of_val(&header.format) +
            std::mem::size_of_val(&header.ntracks) +
            std::mem::size_of_val(&header.tickdiv), 14);
    }

    #[test]
    fn mthd_is_valid() {

        // relying on randomness for a test is bad
        // should be making custom headers to test these things
        // or, better, should be using a seeded random number generator to get predictable results
        // but, because this new function does rely on randomness, we will just loop and make a bunch of them
        for _ in 0..100 {
            let header = MThd::new();

            assert_eq!(header.identifier[0] as char, 'M');
            assert_eq!(header.identifier[1] as char, 'T');
            assert_eq!(header.identifier[2] as char, 'h');
            assert_eq!(header.identifier[3] as char, 'd');

            assert_eq!(header.chunklen, 6);

            assert!(header.format < 3);
        
            if header.format == 0 {
                assert_eq!(header.ntracks, 1);
            }
            else if header.format == 1{
                assert!(header.ntracks >= 2);
            }
            else {
                assert!(header.ntracks >= 1);
            }

            // check 15th bit, could also bitshift
            if (0x8000 & header.tickdiv) == 0 { // bit 15 = 0 means we are using metrical timing
                assert_eq!((header.tickdiv & 0x7FFF), 96); // 96 is a hardcoded value
            }
            else { // bit 15 = 1 means we are using timecode
                // let temp = header.tickdiv & 0xFF00; // unset the lower bits, but we could just shift and be done
                assert!((header.tickdiv >> 8 == 0xE8) || 
                        (header.tickdiv >> 8 == 0xE7) ||
                        (header.tickdiv >> 8 == 0xE3) || 
                        (header.tickdiv >> 8 == 0xE2));
                
                // check lower 8 bits
                assert!((header.tickdiv & 0xFF == 4) ||
                        (header.tickdiv & 0xFF == 8) ||
                        (header.tickdiv & 0xFF == 10) ||
                        (header.tickdiv & 0xFF == 80) ||
                        (header.tickdiv & 0xFF == 100));
            }
        }
    }


}