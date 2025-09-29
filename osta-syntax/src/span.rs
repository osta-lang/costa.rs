pub type SpanRange = std::ops::Range<usize>;

#[derive(Clone, PartialEq)]
pub struct Span(SpanRange);

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self(start..end)
    }

    pub fn start(&self) -> usize {
        self.0.start
    }

    pub fn end(&self) -> usize {
        self.0.end
    }

    pub fn len(&self) -> usize {
        self.0.end - self.0.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn slice<'a>(&self, src: &'a str) -> &'a str {
        &src[self.0.clone()]
    }
}

impl From<SpanRange> for Span {
    fn from(value: SpanRange) -> Self {
        Span(value)
    }
}

impl Into<SpanRange> for Span {
    fn into(self) -> SpanRange {
        self.0
    }
}

impl AsRef<SpanRange> for Span {
    fn as_ref(&self) -> &SpanRange {
        &self.0
    }
}

impl std::ops::Deref for Span {
    type Target = SpanRange;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Span({}..{})", self.start, self.end)
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
