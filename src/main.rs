mod tetris;
use std::io::stdout;
use std::process;
use std::{thread, time};

use std::mem;
use std::sync::Arc;
use std::sync::Mutex;

use crossterm::event::{read, Event::Key, KeyCode};
use crossterm::ExecutableCommand;

use crossterm::{terminal};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream};

use tetris::built_in::built_in;
use tetris::map::Map;
use tetris::pos::Move;

fn byte_to_usize_vec(bytes: &[u8], row_len: usize) -> Vec<Vec<usize>> {
    let col_len = bytes.len() / (row_len * std::mem::size_of::<usize>());
    let mut vec = vec![vec![0usize; row_len]; col_len];

    for i in 0..col_len {
        for j in 0..row_len {
            let offset = (i * row_len + j) * std::mem::size_of::<usize>();
            vec[i][j] = usize::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
        }
    }
    vec
}

fn usize_vec_to_byte(vec: &Vec<Vec<usize>>) -> Option<Vec<u8>> {
    if vec.len() <= 0 {
        return None;
    }
    let mut vec_cpy = vec.clone();
    let _size = mem::size_of::<usize>;
    let flattened: Vec<usize> = vec_cpy.iter().flatten().cloned().collect();

    let byte_array: Vec<u8> = flattened.iter().fold(vec![], |mut acc, &elem| {
        acc.extend(&elem.to_ne_bytes());
        acc
    });

    Some(byte_array)
}

async fn tcp_process(stream: &mut TcpStream, map_data: &Vec<Vec<usize>>) -> Option<Vec<Vec<usize>>> {
    //2차원 벡터 변환
    let byte_array = usize_vec_to_byte(&map_data);
    stream.write_all(&byte_array.unwrap()).await.unwrap();

    //읽기
    let mut buffer = [0; 1024];

    let n = match stream.read(&mut buffer).await {
        Ok(n) if n == 0 => {
            return None;
        }
        Ok(n) => n,
        Err(e) => {
            eprintln!("failed to read from socket; err = {:?}", e);
            return None;
        }
    };
    let data = &buffer[..n];
    let response = byte_to_usize_vec(data, 12);

    Some(response)
}

async fn handle_block(m_mutex: &Arc<Mutex<Map>>, key: KeyCode) {
    let mut map_writer = (*m_mutex).lock().unwrap();

    match key {
        KeyCode::Up => {
            map_writer.spin_block();
        }
        KeyCode::Down => {
            map_writer.down_block();
        }
        KeyCode::Left => {
            map_writer.move_block(Move::Left);
        }
        KeyCode::Right => {
            map_writer.move_block(Move::Right);
        }
        KeyCode::Esc => {
            crossterm::terminal::disable_raw_mode();
            process::exit(0);
        }
        _ => {
            println!("{:?}", key)
        }
    }
}

fn display_game(m_mutex: &Arc<Mutex<Map>>) -> Result<(), ()> {
    let map_data = (*m_mutex).lock().unwrap();
    //let mut map_clone = map_data.map.clone();
    built_in::cls();
    map_data.encoding();
    map_data.print_score();
    Ok(())
}

async fn game_update(m_mutex: &Arc<Mutex<Map>>) -> Result<(), ()> {
    let mut map_writer = (*m_mutex).lock().unwrap();

    map_writer.down_block();
    if map_writer.block.is_none() {
        println!("GAME OVER!");
        return Err(());
    }
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ssstart");

    let map = Arc::new(Mutex::new(Map::new()));

    crossterm::terminal::enable_raw_mode();
    
    let map_clone = Arc::clone(&map);
    let control_thread = tokio::spawn(async move {
        loop {
            match read().unwrap() {
                Key(key) => {
                    handle_block(&map_clone, key.code).await;
                    match game_update(&map_clone).await {
                        Ok(n) => {
                            //서버로 전송
                        }
                        Err(e) => {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    });

    let map_dclone = Arc::clone(&map);
    let update_thread = tokio::spawn(async move {
        loop {
            match game_update(&map_dclone).await {
                Ok(n) => {
                    //서버로 전송
                }
                Err(e) => {
                    break;
                }
            }
            // 1.5초마다 블럭 이동
            thread::sleep(time::Duration::from_millis(1500))
        }
    });

    let map_fclone = Arc::clone(&map);
    let display_thread = tokio::spawn(async move {
        loop {
            built_in::cls();
            display_game(&map_fclone);
            // 1.5초마다 블럭 이동
            thread::sleep(time::Duration::from_millis(100))
        }
    });

    control_thread.await.unwrap();
    update_thread.await.unwrap();
    display_thread.await.unwrap();

    crossterm::terminal::disable_raw_mode();
    Ok(())
}
