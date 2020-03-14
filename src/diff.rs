use std::borrow::Cow;
use std::collections::HashMap;

use difflib::sequencematcher::Match;
use difflib::sequencematcher::SequenceMatcher;

use crate::range::Range;

#[derive(Debug)]
pub enum Author {
    Mine,
    Yours,
    Both,
}

#[derive(Debug)]
pub enum Difference {
    NotChanged(String),
    Add(Author, String),
    Remove(Author, String),
    Modify(Author, String, String),
    Conflict(String, String, String),
}

pub fn diff(base_text: &str, mine_text: &str, yours_text: &str) -> Vec<Difference> {
    let mut munge = Munge::new();
    let base = munge.lines_to_nums(base_text);
    let mine = munge.lines_to_nums(mine_text);
    let yours = munge.lines_to_nums(yours_text);

    let ma = get_matching_blocks(&base, &mine);
    let mb = get_matching_blocks(&base, &yours);

    let (mut ia, mut ib) = (0, 0);
    let mut prev_a_offset = 0;
    let mut prev_b_offset = 0;
    let mut prev_common = Range::START;

    let mut result = Vec::new();
    while ia < ma.len() && ib < mb.len() {
        let a_block = Range(ma[ia].first_start, ma[ia].first_start + ma[ia].size);
        let a_offset = ma[ia].second_start as isize - ma[ia].first_start as isize;
        let b_block = Range(mb[ib].first_start, mb[ib].first_start + mb[ib].size);
        let b_offset = mb[ib].second_start as isize - mb[ib].first_start as isize;

        if let Some(common) = a_block.intersect(b_block) {
            let o = common
                .get_between(prev_common)
                .map(|between| munge.nums_to_lines(&base[between]));
            let a = common
                .transform(a_offset)
                .get_between(prev_common.transform(prev_a_offset))
                .map(|between| munge.nums_to_lines(&mine[between]));
            let b = common
                .transform(b_offset)
                .get_between(prev_common.transform(prev_b_offset))
                .map(|between| munge.nums_to_lines(&yours[between]));

            if o == b && a != b {
                // changes in A
                result.push(detect(Author::Mine, o, a));
            } else if o == a && a != b {
                // changes in B
                result.push(detect(Author::Yours, o, b));
            } else if a != b {
                // conflict
                result.push(Difference::Conflict(
                    o.unwrap_or_default(),
                    a.unwrap_or_default(),
                    b.unwrap_or_default(),
                ));
            } else if o.is_some() || a.is_some() {
                // a == b
                result.push(detect(Author::Both, o, a));
            }

            result.push(Difference::NotChanged(munge.nums_to_lines(&base[common])));

            prev_common = common;
            prev_a_offset = a_offset;
            prev_b_offset = b_offset;
        }

        if a_block.1 < b_block.1 {
            ia += 1;
        } else {
            ib += 1;
        }
    }

    // Remove ending NotChanged
    result.pop();
    return result;

    fn get_matching_blocks(a: &[usize], b: &[usize]) -> Vec<Match> {
        assert!(!a.is_empty() && a[a.len() - 1] == 0);
        assert!(!b.is_empty() && b[b.len() - 1] == 0);

        let mut matcher = SequenceMatcher::new(&a[..(a.len() - 1)], &b[..(b.len() - 1)]);
        let mut matches = matcher.get_matching_blocks();
        matches.pop();
        matches.push(Match {
            first_start: a.len() - 1,
            second_start: b.len() - 1,
            size: 1,
        });
        matches.push(Match {
            first_start: a.len(),
            second_start: b.len(),
            size: 0,
        });
        matches
    }

    fn detect(author: Author, origin: Option<String>, other: Option<String>) -> Difference {
        match (origin.is_some(), other.is_some()) {
            (true, true) => Difference::Modify(author, origin.unwrap(), other.unwrap()),
            (true, false) => Difference::Remove(author, origin.unwrap()),
            (false, true) => Difference::Add(author, other.unwrap()),
            (false, false) => unreachable!(),
        }
    }
}

struct Munge<'a> {
    lines: Vec<Cow<'a, str>>,
    line_hashes: HashMap<&'a str, usize>,
}

impl<'a> Munge<'a> {
    fn new() -> Munge<'a> {
        let mut lines = Vec::new();
        let line_hashes = HashMap::new();

        lines.push(String::new().into());
        Munge { lines, line_hashes }
    }

    fn lines_to_nums(&mut self, text: &'a str) -> Vec<usize> {
        let mut nums: Vec<_> = text
            .lines()
            .map(|line| match self.line_hashes.get(line) {
                Some(i) => *i,
                None => {
                    let next_num = self.lines.len();

                    self.lines.push(line.into());
                    self.line_hashes.insert(line, next_num);
                    next_num
                }
            })
            .collect();
        nums.push(0);
        nums
    }

    fn nums_to_lines(&self, nums: &[usize]) -> String {
        let mut text = String::new();
        for line in nums.iter().map(|&num| &self.lines[num]) {
            text.push_str(line);
            text.push('\n');
        }
        text
    }
}
