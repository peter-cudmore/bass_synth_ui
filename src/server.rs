use std::mem::size_of;
use crate::app::Message;
use crate::bindings::{ParameterType_Waveform, ParameterValue, Section_Osc1, SynthMessage, Patch};

use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::time::Duration;
use zmq;

const ADDRESS: &str = "tcp://bela.local:5555";



pub fn run_server(rx: Receiver<Message>, tx: Sender<Patch>){
    let mut ctx = zmq::Context::new();
    let mut server = ctx.socket(zmq::PAIR).expect("Failed to create socket");
    server.connect(ADDRESS).unwrap();

    'outer: loop {
        match server.recv_bytes(zmq::DONTWAIT) {
            Ok(msg) => {
                if msg.len() == size_of::<Patch>() {
                    let patch = unsafe { *(msg.as_ptr() as *const Patch)};
                    if let Err(_) = tx.send(patch){
                        break 'outer;
                    }
                }
            }
            Err(zmq::Error::EAGAIN) => {},
            Err(e) => {//
                eprintln!("{:?}", e);
               }
        }
        'rx_loop: loop {
            match rx.try_recv() {
                Err(TryRecvError::Disconnected) => {
                    break 'outer;
                }
                Ok(msg) => {
                    let to_osc = SynthMessage::from(msg);
                    let bytes = unsafe{ any_as_u8_slice(&to_osc) };

                    server.send(bytes, zmq::DONTWAIT).expect("Failed to send to server");
                },
                _ => { break 'rx_loop; }
            }
        }

        std::thread::sleep(Duration::from_millis(15));
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        size_of::<T>(),
    )
}

impl From<Message> for SynthMessage {
    fn from(value: Message) -> Self {
        match value {
            Message::SetWaveform(osc, waveform) => {
                SynthMessage {
                    channel: 16,
                    destination: Section_Osc1,
                    parameter: ParameterType_Waveform,
                    value:  ParameterValue{ value_WaveformEnum: waveform }
                }
            },
            Message::SetParameter(section, param_type, value) =>{
                SynthMessage{channel:16, destination:section, parameter: param_type, value}
            }
            _ => {
                SynthMessage { channel: 16, destination: 0, parameter: 0, value: ParameterValue { value_int8_t: 0 } }
            }
        }
    }
}