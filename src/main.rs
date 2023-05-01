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
use tetris::map::{Map, TetrisData};
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

async fn tcp_process(m_mutex: &Arc<Mutex<Map>>) {

    fn to_array<T: Default + Copy, const ROW: usize, const COL: usize>(vec: &Vec<Vec<T>>) -> [[T; COL]; ROW] {
        let mut arr = [[T::default(); COL]; ROW];
        for i in 0..ROW {
            for j in 0..COL {
                arr[i][j] = vec[i][j];
            }
        }
        arr
    }

    let mut stream = TCP_STREAM.get().unwrap().lock().unwrap();

    let mut map_writer = (*m_mutex).lock().unwrap();
    
    //serialize
    let pack = TetrisData {
        score: map_writer.score,
        map: to_array::<usize, 20, 10>(&map_writer.screen)
    };
    let send_data = serde_json::to_string(&pack).unwrap();
    //write
    (*stream).write_all(send_data.as_bytes()).unwrap();
    //read
    let mut buffer = [0; 4096];
    let n = match stream.read(&mut buffer) {
        Ok(n) if n == 0 => {
            return;
        }
        Ok(n) => n,
        Err(e) => {
            eprintln!("failed to read from socket; err = {:?}", e);
            return;
        }
    };
    let data = match std::str::from_utf8(&buffer[..n]) {
        Ok(s) => s.to_owned(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    //deserialize
    let response:TetrisData = serde_json::from_str(&data).unwrap();
    map_writer.gameData.score = response.score;
    map_writer.gameData.map = response.map.clone();

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

async fn display_game(m_mutex: &Arc<Mutex<Map>>) -> Result<(), ()> {
    let map_data = (*m_mutex).lock().unwrap();
    built_in::cls();
    map_data.display();
    map_data.print_score();
    map_data.print_enemy_score();
    Ok(())
}

async fn game_update(m_mutex: &Arc<Mutex<Map>>, down_block: bool) -> Result<(), ()> {
    let mut map_writer = (*m_mutex).lock().unwrap();
    if down_block {
        map_writer.down_block();
    }
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
    println!("connected");
    crossterm::terminal::enable_raw_mode();

    let map_clone = Arc::clone(&map);
    let control_thread = tokio::spawn(async move {
        loop {
            match read().unwrap() {
                Key(key) => {
                    handle_block(&map_clone, key.code).await;
                    game_update(&map_clone, false).await;
                    // tcp_process(&map_clone).await;
                    display_game(&map_clone).await;
                }
                _ => {}
            }
        }
    });

    let map_dclone = Arc::clone(&map);
    let update_thread = tokio::spawn(async move {
        loop {
            let isEnd = game_update(&map_dclone, true).await;
            match isEnd {
                Ok(_)=>{}
                Err(_)=>{
                    crossterm::terminal::disable_raw_mode();
                    std::process::exit(0);
                }
            }
            tcp_process(&map_dclone).await;
            
            // 1.5초마다 블럭 이동
            thread::sleep(time::Duration::from_millis(1500))
        }
    });

    let map_fclone = Arc::clone(&map);
    let display_thread = tokio::spawn(async move {
        loop {
            display_game(&map_fclone).await;
            thread::sleep(time::Duration::from_millis(500))
        }
    });

    control_thread.await.unwrap();
    update_thread.await.unwrap();
    display_thread.await.unwrap();

    crossterm::terminal::disable_raw_mode();
    Ok(())
}
