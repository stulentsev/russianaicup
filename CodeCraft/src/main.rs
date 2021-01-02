mod indexmap;
mod influence;
mod my_strategy;
mod occupancy;
mod pathfinding;
mod quick_start_strategy;
mod vis;

use model::{Action, PlayerView};
use my_strategy::MyStrategy;
use quick_start_strategy::QuickStartStrategy;
use std::time::Instant;

trait GameStrategy {
    fn get_action(
        &mut self,
        player_view: &PlayerView,
        debug_interface: Option<&mut DebugInterface>,
    ) -> Action;
    fn debug_update(&mut self, player_view: &PlayerView, debug_interface: &mut DebugInterface);
}

struct Args {
    host: String,
    port: u16,
    token: String,
    strategy_name: String,
}

impl Args {
    fn parse() -> Self {
        let mut args = std::env::args();
        args.next().unwrap();
        let host = args.next().unwrap_or_else(|| "127.0.0.1".to_owned());
        let port = args
            .next()
            .map_or(31001, |s| s.parse().expect("Can't parse port"));
        let token = args
            .next()
            .unwrap_or_else(|| "0000000000000000".to_string());
        let strategy_name = args.next().unwrap_or_else(|| "main".to_string());

        Self {
            host,
            port,
            token,
            strategy_name,
        }
    }
}

struct Runner {
    reader: Box<dyn std::io::BufRead>,
    writer: Box<dyn std::io::Write>,
}

pub struct DebugInterface<'a> {
    reader: &'a mut dyn std::io::Read,
    writer: &'a mut dyn std::io::Write,
    lines_vertices: Vec<model::ColoredVertex>,
    triangle_vertices: Vec<model::ColoredVertex>,
}

impl DebugInterface<'_> {
    fn real_send(&mut self, command: model::DebugCommand) {
        use trans::Trans;
        model::ClientMessage::DebugMessage { command }
            .write_to(self.writer)
            .expect("Failed to write custom debug data");
        self.writer.flush().expect("Failed to flush");
    }
    fn get_state(&mut self) -> model::DebugState {
        use trans::Trans;
        model::ClientMessage::RequestDebugState {}
            .write_to(self.writer)
            .expect("Failed to write request debug state message");
        self.writer.flush().expect("Failed to flush");
        model::DebugState::read_from(self.reader).expect("Failed to read debug state")
    }

    fn send(&mut self, command: model::DebugCommand) {
        use model::*;

        match command {
            DebugCommand::Add { data } => match data {
                DebugData::Log { .. } | DebugData::PlacedText { .. } => {
                    self.real_send(DebugCommand::Add { data })
                }
                DebugData::Primitives {
                    primitive_type: PrimitiveType::Lines,
                    mut vertices,
                } => self.lines_vertices.append(&mut vertices),
                DebugData::Primitives {
                    primitive_type: PrimitiveType::Triangles,
                    mut vertices,
                } => self.triangle_vertices.append(&mut vertices),
            },
            _ => self.real_send(command),
        }
    }

    fn flush_vertice_buffers(&mut self) {
        let vertices = self.lines_vertices.drain(..).collect();
        self.real_send(model::DebugCommand::Add {
            data: model::DebugData::Primitives {
                primitive_type: model::PrimitiveType::Lines,
                vertices,
            },
        });

        let vertices = self.triangle_vertices.drain(..).collect();
        self.real_send(model::DebugCommand::Add {
            data: model::DebugData::Primitives {
                primitive_type: model::PrimitiveType::Triangles,
                vertices,
            },
        });
    }

    fn fill_cell(&mut self, i: i32, j: i32, color: model::Color) {
        self.send(model::DebugCommand::draw_square(
            model::Vec2F32::from_i32(i, j),
            model::Vec2F32::from_i32(i, j).add_xy(1.0),
            color,
        ))
    }

    fn line_gradient(&mut self, from: model::ColoredVertex, to: model::ColoredVertex) {
        self.send(model::DebugCommand::draw_line_gradient(from, to));
    }

    fn mark_cell(&mut self, i: i32, j: i32, color: model::Color) {
        self.send(model::DebugCommand::draw_square(
            model::Vec2F32::from_i32(i, j).add_xy(0.35),
            model::Vec2F32::from_i32(i + 1, j + 1).add_xy(-0.35),
            color,
        ))
    }

    fn log_text(&mut self, text: String) {
        self.send(model::DebugCommand::Add {
            data: model::DebugData::Log { text },
        })
    }

    fn unit_label(&mut self, world_pos: model::Vec2F32, text: String) {
        self.send(model::DebugCommand::Add {
            data: model::DebugData::PlacedText {
                vertex: model::ColoredVertex {
                    world_pos: Some(world_pos),
                    screen_offset: model::Vec2F32 { x: 20.0, y: 0.0 },
                    color: model::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    },
                },
                text,
                alignment: 0.0,
                size: 40.0,
            },
        })
    }
}

impl Runner {
    fn new(args: &Args) -> std::io::Result<Self> {
        use std::io::Write;
        use trans::Trans;
        let stream = std::net::TcpStream::connect((args.host.as_str(), args.port))?;
        stream.set_nodelay(true)?;
        let stream_clone = stream.try_clone()?;
        let reader = std::io::BufReader::new(stream);
        let mut writer = std::io::BufWriter::new(stream_clone);
        args.token.write_to(&mut writer)?;
        writer.flush()?;

        Ok(Self {
            reader: Box::new(reader),
            writer: Box::new(writer),
        })
    }
    fn debug_interface(&mut self) -> DebugInterface {
        DebugInterface {
            reader: &mut self.reader,
            writer: &mut self.writer,
            lines_vertices: Vec::new(),
            triangle_vertices: Vec::new(),
        }
    }
    fn run(mut self) -> std::io::Result<()> {
        use trans::Trans;

        let args = Args::parse();
        let mut strategy: Box<dyn GameStrategy> = match args.strategy_name.as_str() {
            // "noop" => Box::new(NoopStrategy::new()) as Box<dyn GameStrategy>,
            // "shooter" => Box::new(ShooterStrategy::new()) as Box<dyn GameStrategy>,
            // "sequence_replay" => Box::new(SequenceReplayStrategy::new()) as Box<dyn GameStrategy>,
            "quickstart" => Box::new(QuickStartStrategy::new()) as Box<dyn GameStrategy>,
            _ => Box::new(MyStrategy::new()) as Box<dyn GameStrategy>,
        };

        let mut time_spent_in_action = 0u128;
        let mut time_spent_reading_input = 0u128;
        let mut current_tick = 0;

        loop {
            let now = Instant::now();

            match model::ServerMessage::read_from(&mut self.reader)? {
                model::ServerMessage::GetAction {
                    player_view,
                    debug_available,
                } => {
                    time_spent_reading_input += now.elapsed().as_micros();
                    let now = Instant::now();

                    let mut debug_interface = self.debug_interface();
                    let message = model::ClientMessage::ActionMessage {
                        action: strategy.get_action(
                            &player_view,
                            if debug_available {
                                Some(&mut debug_interface)
                            } else {
                                None
                            },
                        ),
                    };
                    message.write_to(&mut self.writer)?;
                    self.writer.flush()?;

                    time_spent_in_action += now.elapsed().as_micros();
                    current_tick += 1;
                    if current_tick % 200 == 0 {
                        println!("tick {}, spent in action {}ms ({}ms avg), spent reading input {}ms ({}ms avg)",
                                 current_tick,
                                 time_spent_in_action / 1000,
                                 time_spent_in_action / current_tick / 1000,
                                 time_spent_reading_input / 1000,
                                 time_spent_reading_input / current_tick / 1000,
                        );
                    }
                }
                model::ServerMessage::Finish {} => break,
                model::ServerMessage::DebugUpdate { player_view } => {
                    strategy.debug_update(&player_view, &mut self.debug_interface());
                    model::ClientMessage::DebugUpdateDone {}.write_to(&mut self.writer)?;
                    self.writer.flush()?;
                }
            }
        }
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    Runner::new(&Args::parse())?.run()
}
