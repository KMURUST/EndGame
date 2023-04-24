use super::pos::Pos;
use super::pos::Move;
use super::built_in::built_in::make_shape;

#[derive(Debug, Clone)]
pub struct Block {
    pub shape: Vec<Vec<usize>>,
    pub pos: Pos,
    pub id: usize,
    pub rot: usize
}

impl Block {
    pub fn new(id: usize, pos: Option<Pos>, rot: usize) -> Self {
        let pos = pos.unwrap_or_else(||Pos{x:3,y:0});
        let shape = make_shape(id, pos, rot).unwrap();

        Self {
            shape: shape,
            pos: pos,
            id: id,
            rot: rot
        }
    }

    pub fn move_block(&mut self, direction: Move) {
        match direction {
            Move::Left => {
                self.pos.x -= 1;
                match make_shape(self.id, self.pos, self.rot) {
                    Ok(ok) => { self.shape = ok }
                    Err(_) => { self.pos.x += 1 }
                }
            }
            Move::Right => {
                self.pos.x += 1;
                match make_shape(self.id, self.pos, self.rot) {
                    Ok(ok) => { self.shape = ok }
                    Err(_) => { self.pos.x -= 1 }
                }
            }
        }
    }

    pub fn is_none(&self) -> bool {
        match make_shape(0, self.pos, 0) {
            Ok(ok) => { self.shape.clone() == ok }
            Err(_) => { false }
        }
    }

    pub fn spin(&mut self) {
        if self.rot + 1 >= 4 {
            self.rot = 0;
        } else {
            self.rot += 1;
        }
        match make_shape(self.id, self.pos, self.rot) {
            Ok(ok) => { self.shape = ok }
            Err(_) => {
                if self.rot == 0 {
                    self.rot = 3
                } else {
                    self.rot -= 1
                }
            }
        }
    }
}