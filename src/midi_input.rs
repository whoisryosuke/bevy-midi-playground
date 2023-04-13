use bevy::{prelude::*, tasks::IoTaskPool};
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiInputConnection, MidiInputPort};
use std::{borrow::BorrowMut, sync::mpsc};

#[derive(Default, Debug)]
pub enum MidiEvents {
    #[default]
    Pressed,
    Released,
    Holding,
}

// Event for MIDI key input
#[derive(Default)]
pub struct MidiInputKey {
    event: MidiEvents,
    id: u8,
    intensity: u8,
}

pub enum MidiCommand {
    Connect(MidiInputPort),
    Disconnect,
}

type MidiPorts = Vec<(String, MidiInputPort)>;

pub enum MidiResponse {
    AvailablePorts(MidiPorts),
    Input(MidiInputKey),
    Error(String),
}

#[derive(Resource)]
pub struct MidiInput {
    pub commands: Sender<MidiCommand>,
    pub response: Receiver<MidiResponse>,
    pub ports: MidiPorts,
}

impl MidiInput {
    pub fn connect(&self, port: MidiInputPort) {
        self.commands.send(MidiCommand::Connect(port));
    }
}

pub struct MidiInputPlugin;

impl Plugin for MidiInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MidiInputKey>()
            .add_startup_system(setup_midi)
            .add_system(sync_state);
    }
}

pub fn setup_midi(mut commands: Commands) {
    // We create sender/receivers for the commands we send to the MIDI input
    let (command_sender, command_receiver) = crossbeam_channel::unbounded::<MidiCommand>();
    let (result_sender, result_receiver) = crossbeam_channel::unbounded::<MidiResponse>();

    let thread_pool = bevy::tasks::TaskPool::new();
    println!("spawning threads {}", thread_pool.thread_num());
    // let thread_pool = IoTaskPool::get();
    thread_pool
        .spawn(sync_midi_input(command_receiver, result_sender))
        .detach();

    println!("inserting resource");
    commands.insert_resource(MidiInput {
        commands: command_sender,
        response: result_receiver,
        ports: Vec::new(),
    });
}

pub fn sync_state(mut midi_input: ResMut<MidiInput>) {
    while let Ok(response) = midi_input.response.recv() {
        match response {
            MidiResponse::AvailablePorts(ports) => {
                midi_input.ports = ports;
            }
            MidiResponse::Input(_) => {}
            MidiResponse::Error(_) => {}
        }
    }
}

async fn sync_midi_input(
    command_receiver: Receiver<MidiCommand>,
    result_sender: Sender<MidiResponse>,
) -> Result<(), crossbeam_channel::SendError<MidiResponse>> {
    let midi_instance =
        midir::MidiInput::new("midir reading input").expect("Couldn't initialize MidiInput");

    let ports = midi_instance
        .ports()
        .into_iter()
        .map(|port| {
            let name = midi_instance.port_name(&port).unwrap();
            (name, port)
        })
        .collect();
    result_sender.send(MidiResponse::AvailablePorts(ports))?;

    // midi_instance.ignore(midir::Ignore::None);
    // We store the connection to the device here
    // Lets the loop persist below receiving commands without reconnecting everytime
    // let mut midi_input: Option<midir::MidiInput> = Some(midi_instance);
    let mut midi_connection: Option<(MidiInputConnection<()>, MidiInputPort)> = None;

    println!("looping");

    // Listen for commands from app
    // while let Ok(command) = command_receiver.recv() {
    //     println!("Received command");
    //     match command {
    //         MidiCommand::Connect(device_port) => {
    //             // let midi_instance = midir::MidiInput::new("midir reading input")
    //             //     .expect("Couldn't initialize MidiInput");
    //             // // let input = midi_input.unwrap_or_else(|| midi_connection.unwrap().0.close().0);
    //             // let midi_connect_result = midi_instance.connect(
    //             //     &device_port,
    //             //     "midir-read-input",
    //             //     move |stamp, message, _| {
    //             //         println!("{}: {:?} (len = {})", stamp, message, message.len());
    //             //     },
    //             //     (),
    //             // );

    //             // match midi_connect_result {
    //             //     Ok(connection) => {
    //             //         midi_connection = Some((connection, device_port));
    //             //     }
    //             //     Err(error) => {
    //             //         midi_connection = None;
    //             //         println!("Couldn't connect to device: {}", error);
    //             //     }
    //             // }
    //         }
    //         MidiCommand::Disconnect => {
    //             // if let Some((connection, _)) = midi_connection {
    //             //     connection.close();
    //             // }
    //         }
    //     }
    // }
    Ok(())
}
