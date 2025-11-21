use ring::digest;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn hash_file(path: &Path) -> Option<[u8; 32]> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);

    let mut ctx = digest::Context::new(&digest::SHA256);
    let mut buf = [0u8; 8192];

    loop {
        let n = reader.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        ctx.update(&buf[..n]);
    }

    let digest = ctx.finish();
    let mut out = [0u8; 32];
    out.copy_from_slice(digest.as_ref());
    Some(out)
}
