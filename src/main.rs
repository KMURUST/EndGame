mod tetris;
use std::io::stdout;
use std::process;
use std::{thread, time};

use std::mem;
use std::sync::Arc;
use std::sync::Mutex;

use crossterm::event::{read, Event::Key, KeyCode};
use crossterm::ExecutableCommand;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

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

async fn tcp_process(s_mutex: &Arc<Mutex<TcpStream>>, m_mutex: &Arc<Mutex<Map>>) -> Option<Vec<Vec<usize>>> {
    let mut stream = (*s_mutex).lock().unwrap();
    let map_writer = (*m_mutex).lock().unwrap();
    let map_data = map_writer.map.clone();
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

async fn game_update(m_mutex: &Arc<Mutex<Map>>) -> Result<(), ()> {
            
    let mut map_writer = (*m_mutex).lock().unwrap();

    built_in::cls();
    map_writer.down_block();
    map_writer.encoding();
    map_writer.print_score();
    if map_writer.block.is_none() {
        println!("GAME OVER!");
        return Err(());
    }
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ssstart");
    
    let stream = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:8080").await.unwrap()));
    let map = Arc::new(Mutex::new(Map::new()));
    
    crossterm::terminal::enable_raw_mode();
    
    let map_clone = Arc::clone(&map);
    let stream_clone = Arc::clone(&stream);
    let main_thread = tokio::spawn(async move {
        
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
        
        loop {
            match read().unwrap() {
                Key(key) => {
                    handle_block(&map_clone, key.code).await;
                    match game_update(&map_clone).await {
                        Ok(n) => {
                            //서버로 전송
                        },
                        Err(e) => {
                            break ;
                        }
                    }
                }
                _ => {}
            }
            //tcp_process(&stream_clone, &map_clone).await;
        }
    });

    let map_wclone = Arc::clone(&map);
    let stream_wclone = Arc::clone(&stream);
    
    let down_thread = tokio::spawn(async move {

        loop {
            match game_update(&map_wclone).await {
                Ok(n) => {
                    //서버로 전송
                },
                Err(e) => {
                    break ;
                }
            }
            // 1.5초마다 블럭 이동
            thread::sleep(time::Duration::from_millis(1500))
        }
    });

    main_thread.await.unwrap();
    down_thread.await.unwrap();

    crossterm::terminal::disable_raw_mode();
    Ok(())
}
