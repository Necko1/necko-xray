use anyhow::{anyhow, bail, Result};

pub fn parse_seconds(input: &str) -> Result<u64> {
    let mut num = String::new();
    let mut unit = String::new();
    let mut total: u64 = 0;

    fn add_token(num: &str, unit: &str, acc: &mut u64) -> Result<()> {
        if num.is_empty() {
            return Ok(());
        }

        let value: u64 = num.parse()
            .map_err(|e| anyhow!("Invalid number `{num}`: {e}"))?;

        let secs_per_unit: u64 = match unit {
            "" | "s" | "sec" | "secs" | "second" | "seconds" => 1,
            "m" | "min" | "mins" | "minute" | "minutes" => 60,
            "h" | "hr" | "hrs" | "hour" | "hours" => 60 * 60,
            "d" | "day" | "days" => 24 * 60 * 60,
            "w" | "wk" | "wks" | "week" | "weeks" => 7 * 24 * 60 * 60,
            "mo" | "mon" | "mons" | "month" | "months" => 30 * 24 * 60 * 60,
            "q" | "quarter" | "quarters" => 90 * 24 * 60 * 60,
            "y" | "yr" | "yrs" | "year" | "years" => 365 * 24 * 60 * 60,
            _ => bail!("Unsupported time unit `{unit}`"),
        };

        let part = value
            .checked_mul(secs_per_unit)
            .ok_or_else(|| anyhow!("Overflow while multiplying"))?;
        *acc = acc
            .checked_add(part)
            .ok_or_else(|| anyhow!("Overflow while adding"))?;

        Ok(())
    }

    let flush = |acc: &mut u64, num: &mut String, unit: &mut String| -> Result<()> {
        if !num.is_empty() {
            add_token(num, unit, acc)?;
            num.clear();
            unit.clear();
        }
        Ok(())
    };

    for c in input.chars() {
        if c.is_ascii_digit() {
            if !unit.is_empty() {
                flush(&mut total, &mut num, &mut unit)?;
            }
            num.push(c);
        } else if c.is_ascii_alphabetic() {
            unit.push(c.to_ascii_lowercase());
        } else if c.is_whitespace() {
            flush(&mut total, &mut num, &mut unit)?;
        } else {
            bail!("Unexpected character `{c}`");
        }
    }

    flush(&mut total, &mut num, &mut unit)?;
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_seconds_test() {
        assert_eq!(parse_seconds("2s").unwrap(), 2);
        assert_eq!(parse_seconds("1d").unwrap(), 86400);
        assert_eq!(parse_seconds("1day12h").unwrap(), 1*86400 + 12*3600);
        assert_eq!(parse_seconds("30min30sec").unwrap(), 30*60 + 30);
        assert_eq!(parse_seconds("2s 1d 48h 9w").unwrap(), 2 + 1*86400 + 48*3600 + 9*7*86400);
    }
}
