mod config;
use bevy::math::cubic_splines;
use bevy::prelude::*;
use bevy_client_server_events::{
    client::{ConnectToServer, ReceiveFromServer, SendToServer},
    client_server_events_plugin, NetworkConfig,
};
use bevy_flycam::prelude::*;
use config::*;
use lazy_static::lazy_static;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::thread;

lazy_static! {
    // x,y z global variable for cude position
    static ref CUBE_POSITION: Arc<Mutex<(f32, f32, f32, bool)>> = Arc::new(Mutex::new((0.0, 0.0, 0.0, false)));
}

use std::f32::consts::PI;
#[derive(Component)]
struct CubeState {
    start_pos: Vec3,
    move_speed: f32,
    turn_speed: f32,
}
struct NetworkSender {
    pub sender: Sender<String>,
}

impl Resource for NetworkSender {}

struct NetworkReceiver {
    pub receiver: Arc<Mutex<Receiver<String>>>,
}

impl Resource for NetworkReceiver {}

fn main() {
    /*let (tx, rx) = std::sync::mpsc::channel::<String>();
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    socket
        .connect("127.0.0.1:12345")
        .expect("Failed to connect to server");
    let socket_clone = socket.try_clone().expect("Failed to clone socket");
    thread::spawn(move || {

        for message in rx.iter() {
            socket
                .send(message.as_bytes())
                .expect("Failed to send message");
        }
    });

    thread::spawn(move || {
        loop {
            let mut buf = [0; 1024];
            let (amt, _) = socket_clone.recv_from(&mut buf).expect("Failed to receive message");
            let msg = String::from_utf8_lossy(&buf[..amt]);
            println!("Received: {}", msg);

            // Split the message by commas
            let coords: Vec<&str> = msg.split(',').collect();

            // Update the global cube position x nad y
            if coords.len() == 3 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    coords[0].parse::<f32>(),
                    coords[1].parse::<f32>(),
                    coords[2].parse::<f32>(),
                )
                 {
                    let mut cube_position = CUBE_POSITION.lock().unwrap();
                    if cube_position.0 != x {
                        *cube_position = (x, y, z, true);
                        println!("Updated cube position to {:?}", cube_position);
                    }
                }
            }
        }

    });*/

    let mut app = App::new();
    client_server_events_plugin!(
        app,
        MessageToServer => NetworkConfig::default(),
        MessageToClient => NetworkConfig::default()
    );

    app.insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 12.0,          // default: 12.0
        })
        //.insert_resource(NetworkSender { sender: tx })
        .add_systems(Startup, setup)
        .add_systems(Update, send_camera_position_system)
        .add_systems(Update, (process_updates_system, move_players).chain())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut connect_to_server: EventWriter<ConnectToServer>,
) {
    connect_to_server.send(ConnectToServer {
        server_port: SERVER_PORT,
        ..default()
    });
    // plane
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 5.0,
            ..Default::default()
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    },));

    // cube
    /*
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    */
    let cube_spawn = Transform::from_xyz(0.0, 0.5, 0.0);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: cube_spawn,
            ..default()
        },
        CubeState {
            start_pos: cube_spawn.translation,
            move_speed: 2.0,
            turn_speed: 0.2,
        },
    ));

    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    info!("Move camera around by using WASD for lateral movement");
    info!("Use Left Shift and Spacebar for vertical movement");
    info!("Use the mouse to look around");
    info!("Press Esc to hide or show the mouse cursor");
}

fn send_camera_position_system(
    net_sender: Res<NetworkSender>,
    query: Query<&Transform, With<Camera>>,
) {
    for transform in query.iter() {
        let position = transform.translation;
        let message = format!("{},{},{}", position.x, position.y, position.z);
        if let Err(err) = net_sender.sender.send(message) {
            eprintln!("Failed to send message: {}", err);
        }
    }
}

fn move_players(mut cubes: Query<(&mut Transform, &mut CubeState)>, timer: Res<Time>) {
    let mut cube_position = CUBE_POSITION.lock().unwrap();

    if cube_position.3 {
        for (mut transform, cube) in &mut cubes {
            println!("Transform includes {:?}", transform);
            // Move the cube forward smoothly at a given move_speed.
            let forward = transform.forward();
            //transform.translation += forward * cube.move_speed * timer.delta_seconds();
            transform.translation =
                Vec3::new(cube_position.0, cube_position.1, cube_position.2 + 10.0);
            cube_position.3 = false;
        }
    }
}

fn process_updates_system(
    net_receiver: Option<Res<NetworkReceiver>>,
    mut query: Query<&mut Transform>, // , With<Camera>
) {
    if let Some(receiver) = net_receiver {
        if let Ok(rx) = receiver.receiver.lock() {
            if let Ok(update) = rx.try_recv() {
                // Assume the update is in the format "x,y,z"
                let coords: Vec<&str> = update.split(',').collect();
                if coords.len() == 3 {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        coords[0].parse::<f32>(),
                        coords[1].parse::<f32>(),
                        coords[2].parse::<f32>(),
                    ) {
                        for mut transform in query.iter_mut() {
                            transform.translation = Vec3::new(x, y, z);
                            println!("Updated camera position to {:?}", transform.translation);
                        }
                    }
                }
            }
        }
    }
}
