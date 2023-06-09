use rand::{Rng, thread_rng};
use std::io::{stdout};

use crossterm::execute;
use crossterm::style::{Print, SetForegroundColor, SetBackgroundColor, ResetColor, Color};
use super::block::Block;
use super::pos::Move;
use super::built_in::built_in::{make_shape, self, cls};
use std::thread;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TetrisData {
    pub map: [[usize; 10]; 20], // 10x20 map
    pub score: usize, // current score
}

#[derive(Debug, Clone)]
pub struct Map {
    pub map: Vec<Vec<usize>>,
    pub block: Block,
    pub score: usize,
    pub best_score: usize,
    pub stop: bool,

    pub screen: Vec<Vec<usize>>,
    pub gameData: TetrisData
}

/*
# 1
⬜⬜⬜⬜
# 2
⬛🟪
🟪🟪🟪
# 3
🟨🟨
🟨🟨
# 4
🟦
🟦🟦🟦
# 5
⬛⬛🟧
🟧🟧🟧
# 6
🟥🟥
⬛🟥🟥
# 7
⬛🟩🟩
🟩🟩
*/

impl Map {
    
    pub fn new() -> Self {
        let mut map: Vec<Vec<usize>> = vec![];
        
        let mut screen: Vec<Vec<usize>> = vec![];
        
        let mut rng = thread_rng();
        let block:Block = Block::new(rng.gen_range(1..8), None, 0);
        
        fn  read_best_score() -> usize {
            let root = std::env::current_dir().unwrap();
            let dir_path = &root.join("data").to_str().unwrap().replace("\\", "/");
            let dir_path = Path::new(dir_path);
            let path = root.join("data/score.txt").to_str().unwrap().replace("\\","/");
            
            let checker = fs::read_to_string(&path);
            //  create file when read failed
            let best_score: usize = match checker {
                Ok(r) => { r.parse::<usize>().unwrap() },
                Err(e) => {
                    //경로가 존재하지 않으면 directory 생성
                    if !dir_path.exists() {
                        fs::create_dir(dir_path).unwrap();
                    }
                    if e.kind() == std::io::ErrorKind::NotFound {
                        //file이 존재하지 않으면 file 생성 후 0 write
                        let mut file = File::create(path).unwrap();
                        file.write_all("0".as_bytes()).unwrap();
                    }
                    0 as usize
                }
            };
            best_score
        }
        
        // 점수 기록
        let best_score = read_best_score();
        
        map = vec![vec![0; 10]; 20];
        screen = vec![vec![0; 10]; 20];

        let g_data = TetrisData {
            map: [[0usize; 10]; 20],
            score: 0
        };
        
        Self {
            map: map,
            block: block,
            score: 0,
            best_score: best_score,
            stop: false,
            
            screen: screen,
            gameData: g_data
        }
    }

    pub fn set_block(&mut self) {
        let block = self.block.clone();
        self.spawn_block();
        for shape in block.shape {
            self.map[shape[1]][shape[0]] = block.id;
        }
        let mut i: usize = 0;
        let mut earn = 10;
        self.stop = true;
        for map in &self.map.clone() {
            let mut ok: bool = true;
            for i in map {
                if *i == 0 {
                    ok = false;
                    break;
                }
            }
            if ok {
                for j in 0..10 {
                    self.map[i][j] = 0;
                    cls();
                    self.encoding();
                    self.print_score();
                    self.score += earn;
                }
                earn += 10;
                self.map.remove(i);
                self.map.reverse();
                self.map.push(vec![0;10]);
                self.map.reverse();

                if self.score > self.best_score {
                    self.best_score = self.score;
                    // 점수 기록
                    let root = std::env::current_dir().unwrap();
                    let path = root.join("data/score.txt").to_str().unwrap().replace("\\","/");
                    let checker = fs::write(path, self.best_score.to_string());
                    match checker {
                        Ok(ok) => ok,
                        Err(_) => println!("Score Save Failed.")
                    }
                }
            }
            i += 1;
        }
        self.stop = false;
    }    

    pub fn print_score(&self) {
        println!("\r\n");
        let bold = Color::Rgb { r: 138, g: 70, b: 255 };
        let org = Color::Rgb { r: 129, g: 135, b: 251 };
        _ = execute!(
            stdout(),
            SetForegroundColor(Color::Black),
            SetBackgroundColor(org),
            Print("score      ".to_string()),
            ResetColor
        );
        _ = execute!(
            stdout(),
            SetForegroundColor(Color::White),
            SetBackgroundColor(bold),
            Print(self.score.to_string()),
            ResetColor
        );

        print!("\r\n");
        _ = execute!(
            stdout(),
            SetForegroundColor(Color::Black),
            SetBackgroundColor(org),
            Print("best score ".to_string()),
            ResetColor
        );
        _ = execute!(
            stdout(),
            SetForegroundColor(Color::White),
            SetBackgroundColor(bold),
            Print(self.best_score.to_string()),
            ResetColor
        );
    }

    pub fn print_enemy_score(&self) {
        println!("\r\n");
        let bold = Color::Rgb { r: 138, g: 70, b: 255 };
        let org = Color::Rgb { r: 129, g: 135, b: 251 };
        _ = execute!(
            stdout(),
            SetForegroundColor(Color::Black),
            SetBackgroundColor(org),
            Print("enemy score  ".to_string()),
            ResetColor
        );
        _ = execute!(
            stdout(),
            SetForegroundColor(Color::White),
            SetBackgroundColor(bold),
            Print(self.gameData.score.to_string()),
            ResetColor
        );
    }

    pub fn down_block(&mut self){
        if self.stop{return;}
        self.block.pos.y += 1;
        match make_shape(self.block.id, self.block.pos, self.block.rot) {
            Ok(ok) => { 
                let test_block = ok;
                let ok2 = self.check(&test_block);
                if ok2{
                    self.block.shape = test_block.clone();
                }else{
                    self.block.pos.y -= 1;
                    self.set_block();
                }
            }
            Err(_) => { 
                self.block.pos.y -= 1;
                self.set_block();
            }
        };
    }

    pub fn move_block(&mut self, direction: Move){
        if self.stop{return;} 
        let mut block_clone = self.block.clone();
        block_clone.move_block(direction);
        let ok = self.check(&block_clone.shape);
        if ok{
            self.block = block_clone.clone();
        }
    }

    pub fn spin_block(&mut self){
        if self.stop{return;}

        let mut block_clone = self.block.clone();
        block_clone.spin();
        if self.check(&block_clone.shape){
            self.block = block_clone.clone();
        }
    }

    fn spawn_block(&mut self){
        let mut rng = thread_rng();
        let block = Block::new(rng.gen_range(1..8), None, 0);
        for part in &block.shape{
            if self.map[part[1]][part[0]] != 0{
                self.block = Block::new(0, None, 0);
                return;
            }
        }
        self.block = block.clone();
    }

    fn check(&mut self, block_shape: &Vec<Vec<usize>>) -> bool{
        let mut ok: bool = true;
        for part in block_shape{
            if self.map[part[1]][part[0]] != 0{
                ok = false;
                break;
            }
        }
        ok
    }

    pub fn encoding(&mut self) {
        let block = self.block.clone();
        let map = self.map.clone();
        //초기화
        for (i, row) in self.screen.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = map[i][j];
            }
        }
        for shape in &block.shape{
            self.screen[shape[1]][shape[0]] = block.id;
        }
    }

    pub fn display(&self) {

        fn print_row(row:&[usize]) {
            let _ = execute!(
                stdout(),
                SetForegroundColor(Color::White),
                SetBackgroundColor(Color::White),
                Print("ㅤ".to_string()),
                ResetColor
            );

            for j in row {
                let color = match j{
                    0 => Color::Rgb { r: 0, g: 0, b: 0 },
                    1 => Color::Rgb { r: 0, g: 240, b: 240 },
                    2 => Color::Rgb { r: 160, g: 0, b: 240 },
                    3 => Color::Rgb { r: 240, g: 240, b: 0 },
                    4 => Color::Rgb { r: 0, g: 0, b: 240 },
                    5 => Color::Rgb { r: 240, g: 160, b: 240 },
                    6 => Color::Rgb { r: 240, g: 0, b: 0 },
                    _ => Color::Rgb { r: 0, g: 240, b: 0 }
                };
                let b = Color::Rgb{ r: 0, g: 0, b: 0 };

                if color != b {
                    let _ = execute!(
                        stdout(),
                        SetForegroundColor(color),
                        SetBackgroundColor(color),
                        Print("ㅤ".to_string()),
                        ResetColor
                    );
                }else{
                    print!("ㅤ")
                }
            }

            let _ = execute!(
                stdout(),
                SetForegroundColor(Color::White),
                SetBackgroundColor(Color::White),
                Print("ㅤ".to_string()),
                ResetColor
            );
        }
        
        for _ in 0..24{
            let _ = execute!(
                stdout(),
                SetForegroundColor(Color::White),
                SetBackgroundColor(Color::White),
                Print("ㅤ".to_string()),
                ResetColor
            );
        }
        print!("\r\n");

        //game screen
        for i in 0..20 {
            //me
            print_row(&self.screen[i]);
            print_row(&self.gameData.map[i]);
            print!("\r\n");
        }
        
        for _ in 0..24{
            let _ = execute!(
                stdout(),
                SetForegroundColor(Color::White),
                SetBackgroundColor(Color::White),
                Print("ㅤ".to_string()),
                ResetColor
            );
        }
    }

}
