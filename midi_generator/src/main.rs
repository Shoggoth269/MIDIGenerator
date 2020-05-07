extern crate rand;
extern crate rand_distr;
use rand::Rng;
use rand_distr::{Distribution, Normal, NormalError, Uniform};

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
                temp = temp << 8; // set up bits 8 - 14 and shift
                temp = temp | (1 << 15); // because we had to move bit 0 over by 8, bit 7 may have overwritten bit 15 with a 0, let's do this for safety
                temp = temp | match Uniform::from(0..5).sample(&mut rng) as u8 { // set up our sub-frame resolution using the typical values
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
    identifier: String,
    chunklen: u32, // big-endian
    data: Vec<(DeltaTime, Box<dyn Event>)>,
}

impl MTrk {
    fn new() -> MTrk {
        let mut rng = rand::thread_rng();
        let chunky = Uniform::from(1..100);
        let normal = Normal::new(50.0, 25.0).unwrap(); // need proper error handling here
        MTrk {
            identifier: String::from("MTrk"),
            // chunklen: chunky.sample(&mut rng) as u32,
            chunklen: normal.sample(&mut rng) as u32,
            data: Vec::new(),
        }
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    let mut headers: Vec<MThd> = Vec::new();
    for _ in 0..10 {
        headers.push(MThd::new());
    }

    println!("Size of MThd struct: {}", std::mem::size_of::<MThd>());
    for header in headers.iter() {
        // header.identifier.iter().cloned().collect::<String>() // works for a char array, but Rust char is 4 bytes
        println!("{:?}\n\tSize: {}", header, std::mem::size_of_val(&header));
    }

    println!("Size of MThd struct: {:?}\nSize of struct variables:\n\tSize of Identifier: {:?}\n\tSize of chunklen: {}\n\tSize of format: {}\n\tSize of ntracks: {}\n\tSize of tickdiv: {}\n", 
        std::mem::size_of::<MThd>(),
        std::mem::size_of_val(&headers.get(0).unwrap().identifier), 
        std::mem::size_of_val(&headers.get(0).unwrap().chunklen), 
        std::mem::size_of_val(&headers.get(0).unwrap().format), 
        std::mem::size_of_val(&headers.get(0).unwrap().ntracks), 
        std::mem::size_of_val(&headers.get(0).unwrap().tickdiv));

    let trk = MTrk::new();
    println!("{:?}", MTrk::new());
    println!("{:?}", trk);
}
