#![allow(dead_code)]

#[derive(Clone)]
pub struct Gg {
    array: Vec<Vec<CellType>>,
    pub width: usize,
    pub height: usize,
}

#[test]
fn neighbors() {
    let graph = Gg::new(10, 5);
    assert_eq!(
        graph.neighbors(0, 0).sort(),
        vec![(0, 1), (1, 0), (1, 1)].sort()
    );
    assert_eq!(
        graph.neighbors(4, 4).sort(),
        vec![(3, 3), (3, 4), (4, 3), (5, 3), (5, 4)].sort()
    );
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum CellType {
    #[default]
    EmptyCell,
    FullCell,
}

#[derive(Debug)]
pub struct IndexError {
    index: usize,
    limit: usize,
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Index {} out of bounds for axis of size {}",
            self.index, self.limit
        )
    }
}

impl<'a> IntoIterator for &'a Gg {
    type Item = CellType;
    type IntoIter = GgIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        GgIterator {
            gg: &self,
            index_w: 0,
            index_h: 0,
        }
    }
}

impl Gg {
    pub fn new(width: usize, height: usize) -> Gg {
        let array = vec![vec![CellType::EmptyCell; width]; height];
        Gg {
            array,
            width,
            height,
        }
    }

    pub fn neighbors(&self, i: usize, j: usize) -> Vec<(usize, usize)> {
        let a: Vec<usize> = (i as i32 - 1..=i as i32 + 1)
            .into_iter()
            .filter(|i: &i32| 0.le(i) && i32::lt(i, &(self.width as i32)))
            .map(|i| i as usize)
            .collect();

        let b: Vec<usize> = (j as i32 - 1..=j as i32 + 1)
            .into_iter()
            .filter(|j: &i32| 0.le(j) && i32::lt(j, &(self.height as i32)))
            .map(|j| j as usize)
            .collect();

        let mut combinations = Vec::<(usize, usize)>::with_capacity(i * j);
        for aa in &a {
            for bb in &b {
                if *aa == i && *bb == j {
                    continue;
                }
                combinations.push((*aa, *bb));
            }
        }

        combinations
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_val(&self, i: usize, j: usize) -> CellType {
        self.array[j][i]
    }

    pub fn set_val(&mut self, i: usize, j: usize, val: CellType) {
        self.array[j][i] = val;
    }

    pub fn toggle_val(&mut self, i: usize, j: usize) -> std::result::Result<(), IndexError> {
        if i >= self.width {
            return Err(IndexError {
                index: i,
                limit: self.width,
            });
        }
        if j >= self.height {
            return Err(IndexError {
                index: j,
                limit: self.height,
            });
        }
        let val = self.array[j][i];
        self.array[j][i] = match val {
            CellType::FullCell => CellType::EmptyCell,
            CellType::EmptyCell => CellType::FullCell,
        };
        Ok(())
    }

    pub fn iter_mut(&mut self) -> GgIteratorMut {
        GgIteratorMut { gg: self, index: 0 }
    }

    pub fn iter(&self) -> GgIterator {
        GgIterator {
            gg: self,
            index_w: 0,
            index_h: 0,
        }
    }
}

pub struct GgIterator<'b> {
    gg: &'b Gg,
    index_w: usize,
    index_h: usize,
}

impl<'b> Iterator for GgIterator<'b> {
    type Item = CellType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index_w >= self.gg.width {
            self.index_w = 0;
            self.index_h += 1;
        }
        if self.index_h >= self.gg.height {
            return None;
        }
        self.index_w += 1;
        Some(self.gg.array[self.index_h][self.index_w - 1])
    }
}

pub struct GgIteratorMut<'a> {
    gg: &'a mut Gg,
    index: usize,
}

impl<'a> Iterator for GgIteratorMut<'a> {
    type Item = &'a mut CellType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.gg.width * self.gg.height {
            return None;
        }
        let r = self.index % self.gg.width;
        let c = self.index / self.gg.width;
        self.index += 1;
        let row = &mut self.gg.array[c];
        let ptr = row.as_mut_ptr();
        unsafe { Some(&mut *ptr.add(r)) }
    }
}
