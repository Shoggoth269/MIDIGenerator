extern crate rand;
extern crate rand_distr;

use rand::Rng;
use rand_distr::{Distribution, Normal, NormalError, Uniform};

enum MIDIEvents {
    NoteOff,
    NoteOn,
    PolyphonicPressure,
    Controller,
    ProgramChange,
    ChannelPressure,
    PitchBend,
}

#[derive(Debug, Copy, Clone)]
enum MetaEvents {
    SequenceNumber,
    Text,
    Copyright,
    SequenceORTrackName,
    InstrumentName,
    Lyric,
    ProgramName,
    DeviceName,
    MIDIChannelPrefix,
    MIDIPort,
    EndOfTrack,
    SequencerSpecificEvent,
    Marker,
    CuePoint,
    Tempo,
    SMPTEOffset,
    TimeSignature,
    KeySignature,
}

impl MetaEvents {

    /// Returns a random MetaEvents using a Uniform distribution
    /// 
    /// # Arguments
    /// 
    /// * `lower` - A u32 representing the lower bound of the random number generation, minimum value of 0
    /// * `upper` - A u32 representing the upper bound of the random number generation, maximum value of 18
    /// 
    /// To pick between timing events, Lower: 12 and Upper: 18
    fn pick_random(lower: u32, upper: u32) -> MetaEvents {
        let mut rng = rand::thread_rng();
        let test = Uniform::from(lower..upper).sample(&mut rng) as u32;
        match test {
            0 => MetaEvents::SequenceNumber,
            1 => MetaEvents::Text,
            2 => MetaEvents::Copyright,
            3 => MetaEvents::SequenceORTrackName,
            4 => MetaEvents::InstrumentName,
            5 => MetaEvents::Lyric,
            6 => MetaEvents::ProgramName,
            7 => MetaEvents::DeviceName,
            8 => MetaEvents::MIDIChannelPrefix,
            9 => MetaEvents::MIDIPort,
            10 => MetaEvents::EndOfTrack,
            11 => MetaEvents::SequencerSpecificEvent,
            12 => MetaEvents::Marker,
            13 => MetaEvents::CuePoint,
            14 => MetaEvents::Tempo,
            15 => MetaEvents::SMPTEOffset,
            16 => MetaEvents::TimeSignature,
            17 => MetaEvents::KeySignature,
            _ => panic!("Error when picking random event. Number out of bounds.")
        }
    }
}


/* const METAEVENTSLIST: [MetaEvents; 18] = 
                     [ MetaEvents::SequenceNumber, MetaEvents::Text, MetaEvents::Copyright, MetaEvents::SequenceORTrackName,
                       MetaEvents::InstrumentName, MetaEvents::Lyric, MetaEvents::ProgramName, MetaEvents::DeviceName,
                       MetaEvents::MIDIChannelPrefix, MetaEvents::MIDIPort, MetaEvents::EndOfTrack, MetaEvents::SequencerSpecificEvent,
                       MetaEvents::Marker, MetaEvents::CuePoint, MetaEvents::Tempo, MetaEvents::SMPTEOffset,
                       MetaEvents::TimeSignature, MetaEvents::KeySignature ]; */

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
struct DeltaTime {

}

#[derive(Debug)]
struct MidiEvent {

}

#[derive(Debug)]
struct SysExEvent {

}

#[derive(Debug)]
struct MetaEvent {

}

trait Event {
    
}

impl std::fmt::Debug for dyn Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Test")
    }
}

impl Event for MidiEvent {

}

impl Event for SysExEvent {

}

impl Event for MetaEvent {

}

// Track chunk
// A single track chunk will contain a sequence of delta-time / event pairs for chunklen bytes
// The different event types, MidiEvent, SysExEvent, and MetaEvent can all be used in a single track chunk
#[derive(Debug)]
struct MTrk {
    identifier: [u8; 4],
    chunklen: u32, // big-endian
    data: Vec<(DeltaTime, Box<dyn Event>)>,
}

impl MTrk {
    fn new() -> MTrk {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::from(0..3);

        let fmt = uniform.sample(&mut rng) as u16;


        let mut rng = rand::thread_rng();
        let chunky = Uniform::from(1..100);
        // let normal = Normal::new(50.0, 25.0).unwrap(); // need proper error handling here
        // chunklen: chunky.sample(&mut rng) as u32,
        // chunklen: normal.sample(&mut rng) as u32,
        MTrk {
            identifier: ['M' as u8, 'T' as u8, 'r' as u8, 'k' as u8],
            chunklen: 3,
            data: Vec::new(),
        }
    }

    fn new_track_format_0() -> MTrk {
        todo!();
    }

    // global tempo track contains all timing related events and no note data
    // timing events are the following Meta events: Marker, Cue Point, Tempo, SMPTE Offset, Time Signature, and Key Signature
    fn new_global_tempo() -> MTrk {
        let mut rng = rand::thread_rng();
        
        // Generate <DeltaTime>

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

fn main() {
    

    let third_event = MetaEvents::pick_random(0, 18);
    println!("Third Event: {:?}", third_event);


    MetaEvents::pick_random(0, 18);

    let header = MThd::new();
    let mut tracks = Vec::new();

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

    let mut tracks = Vec::new();
    for _ in 0..header.ntracks { // we need to make this many track chunks
        tracks.push(MTrk::new());
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