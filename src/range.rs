use std::ops::Index;

#[derive(Clone, Copy, Debug)]
pub struct Range(pub usize, pub usize);

impl Range {
    pub const START: Range = Range(0, 0);

    pub fn intersect(self, other: Range) -> Option<Range> {
        let left = std::cmp::max(self.0, other.0);
        let right = std::cmp::min(self.1, other.1);

        if left < right {
            Some(Range(left, right))
        } else {
            None
        }
    }

    pub fn contains(self, other: Range) -> bool {
        self.0 <= other.0 && other.1 <= self.1
    }

    pub fn get_between(self, other: Range) -> Option<Range> {
        if self.contains(other) || other.contains(self) {
            None
        } else if self.1 < other.0 {
            Some(Range(self.1, other.0))
        } else if self.0 > other.1 {
            Some(Range(other.1, self.0))
        } else {
            None
        }
    }

    pub fn transform(self, offset: isize) -> Range {
        Range(
            (self.0 as isize + offset) as usize,
            (self.1 as isize + offset) as usize,
        )
    }
}

impl<T> Index<Range> for Vec<T> {
    type Output = [T];

    fn index(&self, range: Range) -> &Self::Output {
        &self[range.0..range.1]
    }
}
