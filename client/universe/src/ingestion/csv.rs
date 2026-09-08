use crate::{Error, invalid};
use std::{collections::BTreeMap, io::BufRead};

/// Bounded RFC4180 records, including quoted newlines and escaped quotes.
/// Memory is proportional to one record, independent of input table length.
pub(super) struct Csv<R> {
    reader: R,
    headers: Vec<String>,
}
const MAX_RECORD: usize = 1024 * 1024;
impl<R: BufRead> Csv<R> {
    pub fn new(mut reader: R, required: &[&str]) -> Result<Self, Error> {
        let headers = record(&mut reader)?.ok_or_else(|| invalid("empty CSV"))?;
        let unique: std::collections::BTreeSet<_> = headers.iter().collect();
        if unique.len() != headers.len()
            || headers.iter().any(String::is_empty)
            || required.iter().any(|key| !headers.iter().any(|h| h == key))
        {
            return Err(invalid("CSV duplicate/empty/missing required column"));
        }
        Ok(Self { reader, headers })
    }
    pub fn next(&mut self) -> Result<Option<BTreeMap<String, String>>, Error> {
        let Some(values) = record(&mut self.reader)? else {
            return Ok(None);
        };
        if values.len() != self.headers.len() {
            return Err(invalid("CSV row width differs from header"));
        }
        Ok(Some(self.headers.iter().cloned().zip(values).collect()))
    }
}
fn record(reader: &mut impl BufRead) -> Result<Option<Vec<String>>, Error> {
    let mut bytes = Vec::new();
    let mut quoted = false;
    loop {
        let available = reader
            .fill_buf()
            .map_err(|e| invalid(format!("CSV read: {e}")))?;
        if available.is_empty() {
            if bytes.is_empty() {
                return Ok(None);
            }
            if quoted {
                return Err(invalid("CSV unterminated quote"));
            }
            break;
        }
        let byte = available[0];
        reader.consume(1);
        if bytes.len() >= MAX_RECORD {
            return Err(invalid("CSV record exceeds 1 MiB"));
        }
        if byte == b'"' {
            quoted = !quoted;
        }
        if byte == b'\n' && !quoted {
            break;
        }
        bytes.push(byte);
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    let row = String::from_utf8(bytes).map_err(|_| invalid("CSV UTF-8"))?;
    let mut chars = row.chars().peekable();
    let mut fields = Vec::new();
    loop {
        let mut field = String::new();
        if chars.peek() == Some(&'"') {
            chars.next();
            loop {
                match chars.next() {
                    Some('"') if chars.peek() == Some(&'"') => {
                        chars.next();
                        field.push('"');
                    }
                    Some('"') => break,
                    Some(c) => field.push(c),
                    None => return Err(invalid("CSV unterminated quoted field")),
                }
            }
        } else {
            while let Some(&c) = chars.peek() {
                if c == ',' {
                    break;
                }
                if c == '"' {
                    return Err(invalid("CSV quote in unquoted field"));
                }
                field.push(c);
                chars.next();
            }
        }
        fields.push(field);
        match chars.next() {
            None => break,
            Some(',') => (),
            _ => return Err(invalid("CSV text after closing quote")),
        }
    }
    Ok(Some(fields))
}

pub(super) fn number(row: &BTreeMap<String, String>, key: &str) -> Result<Option<f64>, Error> {
    let Some(text) = row.get(key).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let n: f64 = text
        .parse()
        .map_err(|_| invalid(format!("invalid numeric column {key}")))?;
    if !n.is_finite() {
        return Err(invalid(format!("nonfinite column {key}")));
    }
    Ok(Some(n))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quoting_width_utf8_and_limits() {
        let mut csv = Csv::new(&b"a,b\r\n\"x,\"\"y\nq\",z\r\n"[..], &["a"]).unwrap();
        assert_eq!(csv.next().unwrap().unwrap()["a"], "x,\"y\nq");
        assert!(csv.next().unwrap().is_none());
        for bad in ["a,a\n", "a,b\nx\n", "a\n\"x\"y\n", "a\nx\"y\n", "a\n\"x\n"] {
            assert!(
                Csv::new(bad.as_bytes(), &["a"])
                    .and_then(|mut c| c.next())
                    .is_err()
            );
        }
        let oversized = format!("a\n{}", "x".repeat(MAX_RECORD + 1));
        assert!(
            Csv::new(oversized.as_bytes(), &["a"])
                .unwrap()
                .next()
                .is_err()
        );
    }
}
