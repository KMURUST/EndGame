pub mod built_in {
    use crossterm::ExecutableCommand;
    use std::mem;
    use std::{io::stdout, thread, time};

    use crate::tetris::pos::Pos;

    pub fn make_shape(id: usize, pos: Pos, rot: usize) -> Result<Vec<Vec<usize>>, ()> {
        let mut block: Vec<Vec<Vec<usize>>> = match id {
            1 => {
                vec![
                    vec![vec![0, 1], vec![1, 1], vec![2, 1], vec![3, 1]],
                    vec![vec![2, 0], vec![2, 1], vec![2, 2], vec![2, 3]],
                    vec![vec![0, 2], vec![1, 2], vec![2, 2], vec![3, 2]],
                    vec![vec![1, 0], vec![1, 1], vec![1, 2], vec![1, 3]],
                ]
            }
            2 => {
                vec![
                    vec![vec![1, 0], vec![1, 1], vec![0, 1], vec![2, 1]],
                    vec![vec![1, 0], vec![1, 1], vec![2, 1], vec![1, 2]],
                    vec![vec![0, 1], vec![1, 1], vec![2, 1], vec![1, 2]],
                    vec![vec![1, 0], vec![1, 1], vec![0, 1], vec![1, 2]],
                ]
            }
            3 => {
                vec![
                    vec![vec![1, 0], vec![2, 0], vec![1, 1], vec![2, 1]],
                    vec![vec![1, 0], vec![2, 0], vec![1, 1], vec![2, 1]],
                    vec![vec![1, 0], vec![2, 0], vec![1, 1], vec![2, 1]],
                    vec![vec![1, 0], vec![2, 0], vec![1, 1], vec![2, 1]],
                ]
            }
            4 => {
                vec![
                    vec![vec![0, 1], vec![1, 1], vec![2, 1], vec![0, 0]],
                    vec![vec![1, 0], vec![1, 1], vec![1, 2], vec![2, 0]],
                    vec![vec![0, 1], vec![1, 1], vec![2, 1], vec![2, 2]],
                    vec![vec![1, 0], vec![1, 1], vec![1, 2], vec![0, 2]],
                ]
            }
            5 => {
                vec![
                    vec![vec![0, 1], vec![1, 1], vec![2, 1], vec![2, 0]],
                    vec![vec![1, 0], vec![1, 1], vec![1, 2], vec![2, 2]],
                    vec![vec![0, 1], vec![1, 1], vec![2, 1], vec![0, 2]],
                    vec![vec![1, 0], vec![1, 1], vec![1, 2], vec![0, 0]],
                ]
            }
            6 => {
                vec![
                    vec![vec![0, 0], vec![1, 1], vec![1, 0], vec![2, 1]],
                    vec![vec![1, 2], vec![1, 1], vec![2, 1], vec![2, 0]],
                    vec![vec![0, 1], vec![1, 1], vec![1, 2], vec![2, 2]],
                    vec![vec![1, 0], vec![1, 1], vec![0, 1], vec![0, 2]],
                ]
            }
            7 => {
                vec![
                    vec![vec![0, 2], vec![1, 1], vec![1, 2], vec![2, 1]],
                    vec![vec![0, 0], vec![1, 1], vec![0, 1], vec![1, 2]],
                    vec![vec![0, 1], vec![1, 1], vec![1, 0], vec![2, 0]],
                    vec![vec![1, 0], vec![1, 1], vec![2, 1], vec![2, 2]],
                ]
            }
            _ => {
                vec![vec![vec![0; 2]; 4]; 4]
            }
        };
        let block_clone = block.clone();
        let mut i: usize = 0;

        let mut ok = true;

        let mut x = 0;
        let mut y = 0;

        for _ in &block_clone[rot] {
            x = block[rot][i][0] as isize;
            y = block[rot][i][1] as isize;
            x += pos.x as isize;
            y += pos.y as isize;

            i += 1;
            if id == 1 {
                if y - 1 < 0 {
                } else {
                    y -= 1;
                }
            }
            if x >= 10 || x < 0 || y >= 20 {
                ok = false;
                break;
            }
        }

        i = 0;
        if ok {
            for _ in &block_clone[rot] {
                if pos.x < 0 {
                    block[rot][i][0] = ((block[rot][i][0] as isize) + pos.x) as usize;
                } else {
                    block[rot][i][0] += pos.x as usize;
                }

                if pos.y < 0 {
                    block[rot][i][1] = ((block[rot][i][1] as isize) + pos.y) as usize;
                } else {
                    block[rot][i][1] += pos.y as usize;
                }
                if id == 1 {
                    if (block[rot][i][1] as isize) - 1 < 0 {
                        drop(block[rot][i][1])
                    } else {
                        block[rot][i][1] -= 1;
                    }
                }
                i += 1;
            }
            Ok(block[rot].clone())
        } else {
            Err(())
        }
    }

    pub fn byte_to_usize_vec(bytes: &[u8], row_len: usize) -> Vec<Vec<usize>> {
        let mut vec = vec![vec![0usize; row_len]; 20];
    
        for i in 0..20 {
            for j in 0..row_len {
            let offset = i * row_len + j;
            vec[i][j] = bytes[offset] as usize;
        }
        }
        vec
    }

    pub fn usize_vec_to_byte(vec: &Vec<Vec<usize>>) -> Option<Vec<u8>> {
        if vec.len() <= 0 {
            return None;
        }
        let vec_cpy = vec.clone();
        let mut byte_array:Vec<u8> = Vec::new();

        for row in vec_cpy {
            for val in row {
                byte_array.push(val as u8);
            }
        }

        Some(byte_array)
    }

    pub fn cls() {
        stdout().execute(crossterm::cursor::MoveTo(0, 0)).unwrap();
        stdout()
            .execute(crossterm::terminal::Clear(
                crossterm::terminal::ClearType::All,
            ))
            .unwrap();
    }
}
