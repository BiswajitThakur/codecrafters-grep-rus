use std::borrow::Cow;

#[derive(Debug, Clone)]
enum PatternToken {
    Char(u8),
    Digit,
    Word,
    Group(Vec<u8>),
    NegGroup(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct Pattern<'a> {
    tokens: Cow<'a, [PatternToken]>,
    start: bool,
    end: bool,
}

impl<'a> Pattern<'a> {
    pub fn compile<T: AsRef<str>>(value: T) -> Self {
        let mut chars = value.as_ref().as_bytes().iter().peekable();
        let mut tokens = Vec::new();
        let mut start = false;
        let mut end = false;
        if chars.peek() == Some(&&b'^') {
            chars.next();
            start = true;
        }
        while let Some(ch) = chars.next() {
            match ch {
                b'\\' if chars.next_if(|&v| *v == b'd').is_some() => {
                    tokens.push(PatternToken::Digit);
                }
                b'\\' if chars.next_if(|&v| *v == b'w').is_some() => {
                    tokens.push(PatternToken::Word);
                }
                b'[' => {
                    let mut vals = Vec::<u8>::new();
                    let mut is_neg_group = false;
                    if let Some(v) = chars.next() {
                        if *v == b'^' {
                            is_neg_group = true;
                        } else {
                            vals.push(*v);
                        }
                    }
                    while let Some(v) = chars.next() {
                        match *v {
                            b']' => break,
                            c => vals.push(c),
                        }
                    }
                    vals.shrink_to_fit();
                    if is_neg_group {
                        tokens.push(PatternToken::NegGroup(vals));
                    } else {
                        tokens.push(PatternToken::Group(vals));
                    }
                }
                b'$' if chars.peek().is_none() => {
                    end = true;
                }
                v => {
                    tokens.push(PatternToken::Char(*v));
                }
            }
        }
        tokens.shrink_to_fit();
        Self {
            tokens: Cow::Owned(tokens),
            start,
            end,
        }
    }
    pub fn test<T: AsRef<str>>(&self, value: T) -> bool {
        let value = value.as_ref().as_bytes();
        if self.start {
            return self.match_here(value);
        }
        let mut i = 0;
        while i < value.len() {
            if self.match_here(&value[i..]) {
                return true;
            }
            i += 1;
        }
        false
    }
    fn match_here(&self, value: &[u8]) -> bool {
        let mut tokens = self.tokens.iter();
        let mut chars = value.iter();
        while let Some(token) = tokens.next() {
            match token {
                PatternToken::Char(ch) => {
                    if chars.next() != Some(ch) {
                        return false;
                    }
                }
                PatternToken::Digit => {
                    let next = chars.next();
                    if next.is_none() {
                        return false;
                    }
                    if !matches!(*next.unwrap(), b'0'..=b'9') {
                        return false;
                    }
                }
                PatternToken::Word => {
                    let next = chars.next();
                    if next.is_none() {
                        return false;
                    }
                    if !matches!(*next.unwrap(), b'a'..=b'z'| b'A'..=b'Z' | b'0'..=b'9') {
                        return false;
                    }
                }
                PatternToken::Group(v) => {
                    if v.is_empty() {
                        continue;
                    }
                    let next = chars.next();
                    if next.is_none() {
                        return false;
                    }
                    if !v.contains(next.unwrap()) {
                        return false;
                    }
                }
                PatternToken::NegGroup(v) => {
                    if v.is_empty() {
                        continue;
                    }
                    let next = chars.next();
                    if next.is_none() {
                        return false;
                    }
                    if v.contains(next.unwrap()) {
                        return false;
                    }
                }
            }
        }
        if self.end && chars.next().is_some() {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    #[test]
    fn test_char_match() {
        let regex = Pattern::compile(r"a");
        assert!(regex.test("a"));
        assert!(regex.test("ba"));
        assert!(regex.test("bcdea"));
        assert!(regex.test("bcade"));
        assert!(regex.test("123a067"));
        let regex = Pattern::compile(r"abc");
        assert!(regex.test("abc"));
        assert!(regex.test("aabcc"));
        assert!(regex.test("xyzabc"));
        assert!(regex.test("abcxyz"));
        assert!(regex.test("xyzabcxyz"));
    }
    #[test]
    fn test_char_not_match() {
        let regex = Pattern::compile(r"a");
        assert!(!regex.test("A"));
        assert!(!regex.test("b"));
        assert!(!regex.test("bcde"));
        assert!(!regex.test("123067"));
        let regex = Pattern::compile(r"abc");
        assert!(!regex.test("ABC"));
        assert!(!regex.test("aabCc"));
        assert!(!regex.test("xyzaBc"));
        assert!(!regex.test("Abcxyz"));
        assert!(!regex.test("xyzabCxyz"));
    }
}
