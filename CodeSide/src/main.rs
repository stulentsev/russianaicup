use std::fmt;
use std::io::{self, BufReader, BufWriter, Write};
use std::net::TcpStream;

use model::*;
use trans::Trans;

mod main_strategy;
mod noop_strategy;
mod shooter_strategy;
mod sequence_replay_strategy;
mod pathfinding;
mod indexmap;

use std::collections::hash_map::RandomState;

mod constants;

use main_strategy::MainStrategy;
use noop_strategy::NoopStrategy;
use shooter_strategy::ShooterStrategy;
use sequence_replay_strategy::SequenceReplayStrategy;
use std::process::exit;

#[derive(Debug)]
struct Args {
    host: String,
    port: u16,
    token: String,
    strategy_name: String,
}

trait GameStrategy {
    fn get_action(&mut self, unit: &Unit, game: &Game, debug: &mut crate::DrawDebug) -> UnitAction;
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

pub struct DrawDebug<'a>(&'a mut dyn io::Write);

impl DrawDebug<'_> {
    fn log(&mut self, args: fmt::Arguments) {
        self.draw(CustomData::Log {
            text: format!("{}", args),
        })
    }

    fn line(
        &mut self,
        p1: impl Into<Vec2F32>,
        p2: impl Into<Vec2F32>,
        width: f32,
        color: ColorF32,
    ) {
        self.draw(CustomData::Line {
            p1: p1.into(),
            p2: p2.into(),
            width,
            color,
        })
    }

    fn rect(&mut self, pos: impl Into<Vec2F32>, size: impl Into<Vec2F32>, color: ColorF32) {
        self.draw(CustomData::Rect {
            pos: pos.into(),
            size: size.into(),
            color,
        })
    }

    fn bbox(&mut self, bbox: BoundingBox, color: ColorF32) {
        self.rect(bbox.bottom_left, bbox.size, color)
    }

    fn draw(&mut self, data: CustomData) {
        PlayerMessageGame::CustomDataMessage { data }
            .write_to(&mut self.0)
            .expect("Failed to write custom debug data");
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let stream = TcpStream::connect((args.host.as_str(), args.port))?;
    stream.set_nodelay(true)?;
    let stream_clone = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let mut writer = BufWriter::new(stream_clone);
    args.token.write_to(&mut writer)?;
    writer.flush()?;

    let mut strategy = match args.strategy_name.as_str() {
        "noop" => Box::new(NoopStrategy::new()) as Box<dyn GameStrategy>,
        "shooter" => Box::new(ShooterStrategy::new()) as Box<dyn GameStrategy>,
        "sequence_replay" => Box::new(SequenceReplayStrategy::new()) as Box<dyn GameStrategy>,
        _ => Box::new(MainStrategy::new()) as Box<dyn GameStrategy>,
    };
    let mut server_msg = ServerMessageGame::read_from(&mut reader)?;

    while let Some(player_view) = server_msg.player_view {
        let action = player_view
            .game
            .units
            .iter()
            .filter(|unit| unit.player_id == player_view.my_id)
            .map(|unit| {
                (
                    unit.id,
                    strategy.get_action(unit, &player_view.game, &mut DrawDebug(&mut writer)),
                )
            })
            .collect();
        PlayerMessageGame::ActionMessage {
            action: Versioned { inner: action },
        }
            .write_to(&mut writer)?;
        writer.flush()?;
        server_msg = ServerMessageGame::read_from(&mut reader)?;
    }
    Ok(())
}
