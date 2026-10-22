//! Exact base-10 values. Binary floating point is not a measurement representation.
use crate::error::DataError;

const MAX_DIGITS: usize = 38;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Digits {
    negative: bool,
    digits: Vec<u8>,
    scale: usize,
}

pub fn canonical_decimal(raw: &str) -> Result<String, DataError> {
    Ok(parse(raw)?.canonical())
}

pub fn multiply_decimals(left: &str, right: &str) -> Result<String, DataError> {
    let left = parse(left)?;
    let right = parse(right)?;
    let mut acc = vec![0u32; left.digits.len() + right.digits.len()];
    for (i, a) in left.digits.iter().rev().enumerate() {
        for (j, b) in right.digits.iter().rev().enumerate() {
            acc[i + j] += u32::from(*a) * u32::from(*b);
        }
    }
    for index in 0..acc.len() - 1 {
        acc[index + 1] += acc[index] / 10;
        acc[index] %= 10;
    }
    if acc.last().is_some_and(|digit| *digit > 9) {
        return Err(DataError::invalid(
            "Decimal product exceeds the supported 38-digit representation.",
        ));
    }
    while acc.len() > 1 && acc.last() == Some(&0) {
        acc.pop();
    }
    acc.reverse();
    let product = Digits {
        negative: left.negative ^ right.negative,
        digits: acc.into_iter().map(|digit| digit as u8).collect(),
        scale: left.scale + right.scale,
    };
    product.canonical_checked()
}

fn parse(raw: &str) -> Result<Digits, DataError> {
    let text = raw.trim();
    if text.is_empty() || text.len() > 64 {
        return Err(DataError::invalid(
            "A measurement value must be a plain decimal of at most 64 characters.",
        ));
    }
    let (negative, rest) = if let Some(rest) = text.strip_prefix('+') {
        (false, rest)
    } else if let Some(rest) = text.strip_prefix('-') {
        (true, rest)
    } else {
        (false, text)
    };
    if rest.is_empty() || rest.contains(['e', 'E', '+', '-']) {
        return Err(DataError::invalid(
            "Scientific notation and unit suffixes are not accepted as measurement values.",
        ));
    }
    let (integer, fraction) = match rest.split_once('.') {
        Some((integer, fraction)) => {
            if integer.is_empty() || fraction.is_empty() {
                return Err(DataError::invalid(
                    "A decimal point must have digits on both sides.",
                ));
            }
            (integer, Some(fraction))
        }
        None => (rest, None),
    };
    if !integer.bytes().all(|byte| byte.is_ascii_digit())
        || fraction
            .is_some_and(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(DataError::invalid(
            "A measurement value must contain only decimal digits.",
        ));
    }
    let digits: Vec<u8> = integer
        .bytes()
        .chain(fraction.unwrap_or("").bytes())
        .map(|byte| byte - b'0')
        .collect();
    if digits.len() > MAX_DIGITS {
        return Err(DataError::invalid(
            "Decimal exceeds the supported 38-digit representation.",
        ));
    }
    let scale = fraction.map_or(0, str::len);
    let negative = negative && digits.iter().any(|digit| *digit != 0);
    Ok(Digits {
        negative,
        digits,
        scale,
    })
}

impl Digits {
    fn canonical(&self) -> String {
        self.canonical_checked()
            .expect("parsed decimals fit the representation")
    }

    fn canonical_checked(&self) -> Result<String, DataError> {
        let mut digits = self.digits.clone();
        while digits.len() > self.scale + 1 && digits.first() == Some(&0) {
            digits.remove(0);
        }
        let mut scale = self.scale;
        while scale > 0 && digits.last() == Some(&0) {
            digits.pop();
            scale -= 1;
        }
        while digits.first() == Some(&0) && digits.len() > scale + 1 {
            digits.remove(0);
        }
        if digits.iter().all(|digit| *digit == 0) {
            return Ok("0".to_owned());
        }
        if digits.len() > MAX_DIGITS {
            return Err(DataError::invalid(
                "Decimal exceeds the supported 38-digit representation.",
            ));
        }
        let integer_len = digits.len() - scale;
        let mut text = String::new();
        if self.negative {
            text.push('-');
        }
        if integer_len == 0 {
            text.push('0');
        } else {
            for digit in &digits[..integer_len] {
                text.push(char::from(b'0' + digit));
            }
        }
        if scale > 0 {
            text.push('.');
            for digit in &digits[integer_len..] {
                text.push(char::from(b'0' + digit));
            }
        }
        Ok(text)
    }
}
