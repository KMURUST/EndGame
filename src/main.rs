mod tetris;

use std::io::{Read, Write};
use std::process;
use std::{thread, time};

use std::sync::Arc;
use std::sync::Mutex;

use crossterm::event::{read, Event::Key, KeyCode};

//use tokio::io::{AsyncReadExt, AsyncWriteExt};
//use tokio::net::TcpStream;
use tokio::sync::OnceCell;

use std::net::{SocketAddr, TcpStream};

use tetris::built_in::built_in;
use tetris::map::Map;
use tetris::pos::Move;

use lazy_static::lazy_static;

lazy_static! {
    static ref TCP_STREAM: OnceCell<Arc<Mutex<TcpStream>>> = OnceCell::new();
}

async fn connect_async(addr: SocketAddr) -> std::io::Result<TcpStream> {
    TcpStream::connect(addr)
}

async fn initialize() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let stream = connect_async(addr).await?;
    TCP_STREAM.set(Arc::new(Mutex::new(stream))).unwrap();
    Ok(())
}
/*
async fn tcp_process(m_mutex: &Arc<Mutex<Map>>) -> Option<Vec<Vec<usize>>> {
    let mut stream = TCP_STREAM.get().unwrap().lock().unwrap();

    let map_writer = (*m_mutex).lock().unwrap();
    //2차원 벡터 변환
    let byte_array = built_in::usize_vec_to_byte(&map_writer.screen);
    (*stream).write_all(&byte_array.unwrap()).unwrap();
    //읽기
    let mut buffer = [0; 1024];
    /*
    let n = match stream.read(&mut buffer) {
        Ok(n) if n == 0 => {
            return None;
        }
        Ok(n) => n,
        Err(e) => {
            eprintln!("failed to read from socket; err = {:?}", e);
            return None;
        }
    };
    */
    let n = 24;
    //let data = &buffer[..n];
    let data = &buffer;
    let response = built_in::byte_to_usize_vec(data, 12);
    Some(response)
}
*/
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

async fn display_game(m_mutex: &Arc<Mutex<Map>>) -> Result<(), ()> {
    let map_data = (*m_mutex).lock().unwrap();
    built_in::cls();
    map_data.display();
    map_data.print_score();
    map_data.print_enemy_score();
    Ok(())
}

async fn game_update(m_mutex: &Arc<Mutex<Map>>) -> Result<(), ()> {
    let mut map_writer = (*m_mutex).lock().unwrap();
    map_writer.down_block();
    if map_writer.block.is_none() {
        println!("GAME OVER!");
        return Err(());
    }
    map_writer.encoding();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ssstart");
    let map = Arc::new(Mutex::new(Map::new()));
    initialize().await?;

    crossterm::terminal::enable_raw_mode();

    let map_clone = Arc::clone(&map);
    let control_thread = tokio::spawn(async move {
        loop {
            match read().unwrap() {
                Key(key) => {
                    handle_block(&map_clone, key.code).await;
                    game_update(&map_clone).await;
                    //tcp_process(&map_clone).await;
                }
                _ => {}
            }
        }
    });

    let map_dclone = Arc::clone(&map);
    let update_thread = tokio::spawn(async move {
        loop {
            game_update(&map_dclone).await;
            //tcp_process(&map_dclone).await;
            
            // 1.5초마다 블럭 이동
            thread::sleep(time::Duration::from_millis(1500))
        }
    });

    let map_fclone = Arc::clone(&map);
    let display_thread = tokio::spawn(async move {
        loop {
            display_game(&map_fclone).await;
            thread::sleep(time::Duration::from_millis(100))
        }
    });

    control_thread.await.unwrap();
    update_thread.await.unwrap();
    display_thread.await.unwrap();

    crossterm::terminal::disable_raw_mode();
    Ok(())
}
