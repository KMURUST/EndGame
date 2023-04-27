mod tetris;
use std::io::stdout;
use std::{thread, time};
use std::process;

use crossterm::ExecutableCommand;
use crossterm::event::{read, Event::Key, KeyCode};

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use tetris::built_in::built_in;
use tetris::map::Map;
use tetris::pos::Move;
fn main() {
    let map = std::sync::Mutex::new(Map::new());
    crossterm::terminal::enable_raw_mode();

    thread::scope(|scope| {
        let main_thread = scope.spawn( || {
            loop {
                let checker = map.lock();
                match checker {
                    Ok(x) => {
                        if !x.stop {
                            built_in::cls();
                            x.encoding();
                            x.print_score();
                            if x.block.is_none() {
                                println!("GAME OVER!");
                                return;
                            }
                        }
                    },
                    Err(e) => { println!("{e:?}") }
                }

                match read().unwrap() {
                    Key(key) => {
                        let mut map_writer = map.lock().unwrap();
                        match key.code {
                            KeyCode::Up => { map_writer.spin_block(); },
                            KeyCode::Down => { map_writer.down_block(); },
                            KeyCode::Left => { map_writer.move_block(Move::Left); },
                            KeyCode::Right => { map_writer.move_block(Move::Right); },
                            KeyCode::Esc => {
                                crossterm::terminal::disable_raw_mode();
                                process::exit(0);
                            },
                            _ => { println!("{:?}", key.code) }
                        }
                    },
                    _ => { }
                }
            }
        });

        let down_thread = scope.spawn(|| {
            loop {
                let checker = map.lock();
                match checker {
                    Ok(x) => {
                        built_in::cls();
                        let mut map_writer = x;
                        map_writer.down_block();
                        map_writer.encoding();
                        map_writer.print_score();
                        if map_writer.block.is_none() {
                            println!("GAME OVER!");
                            return;
                        }
                    },
                    Err(e) => { println!("{e:?}") }
                }
                // 1.5초마다 블럭 이동
                thread::sleep(time::Duration::from_millis(1500))
            }
        });
        
        down_thread.join().unwrap();
        main_thread.join().unwrap();

    });
    crossterm::terminal::disable_raw_mode();
}
