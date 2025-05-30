use rustc_hash::FxHashMap;
use std::io::{self, BufRead};

pub struct ByteLineReader<R: io::Read> {
    data: io::BufReader<R>,
}

impl<R: io::Read> ByteLineReader<R> {
    pub fn new(data: io::BufReader<R>) -> Self {
        Self { data }
    }
}

impl<R: io::Read> Iterator for ByteLineReader<R> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = Vec::new();
        match self.data.read_until(b'\n', &mut buf) {
            Err(e) => {
                log::error!("Error while reading graph: {:?} ", e);
                Some(buf)
            }
            Ok(1..) => Some(buf),
            Ok(0) => None,
        }
    }
}

pub fn transform_walks(line: Vec<u8>, walk_map: &mut FxHashMap<Vec<u8>, Vec<u8>>) -> Vec<u8> {
    match line.first() {
        Some(b'W') => {
            let mut it = line[..].split(|x| x == &b'\t');
            // discard 'W'
            it.next();
            let sample_id = it
                .next()
                .expect("premature termination of W-line: must have sample identifier field");
            let haplo_id = it
                .next()
                .expect("premature termination of W-line: must have haplotype identifier field");
            let seq_id = it
                .next()
                .expect("premature termination of W-line: must have sequence identifier field");
            let seq_start = it
                .next()
                .expect("premature termination of W-line: must have sequence start position field");
            let seq_end = it.next().expect(
                "premature termination of W-line: must have sequence start end position field",
            );

            let mut orig = Vec::new();
            orig.extend_from_slice(sample_id);
            orig.push(b'\t');
            orig.extend_from_slice(haplo_id);
            orig.push(b'\t');
            orig.extend_from_slice(seq_id);
            orig.push(b'\t');
            orig.extend_from_slice(seq_start);
            orig.push(b'\t');
            orig.extend_from_slice(seq_end);

            let mut path_name: Vec<u8> = Vec::new();
            path_name.extend_from_slice(sample_id);
            path_name.push(b'#');
            path_name.extend_from_slice(haplo_id);
            path_name.push(b'#');
            path_name.extend_from_slice(seq_id);
            path_name.push(b':');
            path_name.extend_from_slice(seq_start);
            path_name.push(b'-');
            path_name.extend_from_slice(seq_end);

            // create an ID that is unique to this particular walk, *just in case* a path with the
            // same signature already exists (which is highly unlikely, but hey, let's be sure!
            path_name.extend_from_slice(b"$gfaffix");

            walk_map.insert(path_name.clone(), orig);

            let walk = it
                .next()
                .expect("premature termination of W-line: must have walk field");

            let mut path = Vec::new();
            let mut i = 0;
            let mut p = 0;
            let mut it = walk.iter();
            while let Some(j) = it.position(|x| x == &b'>' || x == &b'<') {
                if j > 0 {
                    path.extend_from_slice(&walk[i + 1..p + j]);
                    match walk[i] {
                        b'>' => path.push(b'+'),
                        b'<' => path.push(b'-'),
                        _ => unreachable!("we only stop at < and >, but observed '{}' at position i={}, j={} at {} so nothing to worry about", walk[i] as char, i, j, std::str::from_utf8(&walk[..j]).unwrap()),
                    }
                    path.push(b',');
                    i = p + j;
                }
                p += j + 1;
            }
            if i < walk.len() {
                let mut end = walk.len();
                if walk[end - 1] == b'\n' {
                    end -= 1
                }
                path.extend_from_slice(&walk[i + 1..end]);
                match walk[i] {
                    b'>' => path.push(b'+'),
                    b'<' => path.push(b'-'),
                    _ => panic!(
                        "expected < or >, but observed '{}' at position i={}",
                        walk[i] as char, i
                    ),
                }
            }
            let mut transformed: Vec<u8> = Vec::new();
            transformed.push(b'P');
            transformed.push(b'\t');
            transformed.append(&mut path_name);
            transformed.push(b'\t');
            transformed.append(&mut path);
            transformed.push(b'\t');
            transformed.push(b'*');
            transformed
        }
        _ => line,
    }
}
