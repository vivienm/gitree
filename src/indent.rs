use std::fmt;
use std::marker::PhantomData;
use std::ops::DerefMut;

pub trait IndentationMarks {
    const TAB: &'static str;
    const BAR: &'static str;
    const TEE: &'static str;
    const ELL: &'static str;
}

pub struct AsciiMarks;

impl IndentationMarks for AsciiMarks {
    const TAB: &'static str = "    ";
    const BAR: &'static str = "|   ";
    const TEE: &'static str = "|-- ";
    const ELL: &'static str = "`-- ";
}

pub struct UnicodeMarks;

impl IndentationMarks for UnicodeMarks {
    const TAB: &'static str = AsciiMarks::TAB;
    const BAR: &'static str = AsciiMarks::BAR;
    const TEE: &'static str = "├── ";
    const ELL: &'static str = "└── ";
}

pub trait IndentationLevel: fmt::Display {
    fn indent(&mut self);

    fn dedent(&mut self);

    fn set_last(&mut self);

    fn is_empty(&self) -> bool;
}

pub struct TreeLevel<M> {
    items: Vec<bool>,
    phantom: PhantomData<M>,
}

impl<M> Default for TreeLevel<M> {
    fn default() -> Self {
        Self::with_capacity(16)
    }
}

impl<M> TreeLevel<M> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        TreeLevel {
            items: Vec::with_capacity(capacity),
            phantom: PhantomData,
        }
    }
}

impl<M> fmt::Display for TreeLevel<M>
where
    M: IndentationMarks,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((child, ancestors)) = self.items.split_last() {
            for ancestor in ancestors {
                write!(f, "{}", if *ancestor { M::TAB } else { M::BAR })?;
            }
            write!(f, "{}", if *child { M::ELL } else { M::TEE })?;
        }
        Ok(())
    }
}

impl<M> IndentationLevel for TreeLevel<M>
where
    M: IndentationMarks,
{
    #[inline]
    fn indent(&mut self) {
        self.items.push(false);
    }

    #[inline]
    fn dedent(&mut self) {
        self.items.pop();
    }

    #[inline]
    fn set_last(&mut self) {
        *self.items.last_mut().unwrap() = true;
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

pub struct NullLevel {
    empty: bool,
}

impl Default for NullLevel {
    fn default() -> Self {
        NullLevel { empty: true }
    }
}

impl NullLevel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl fmt::Display for NullLevel {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl IndentationLevel for NullLevel {
    #[inline]
    fn indent(&mut self) {
        self.empty = false;
    }

    #[inline]
    fn dedent(&mut self) {}

    #[inline]
    fn set_last(&mut self) {}

    #[inline]
    fn is_empty(&self) -> bool {
        self.empty
    }
}

impl<T: DerefMut<Target = dyn IndentationLevel> + fmt::Display> IndentationLevel for T {
    #[inline]
    fn indent(&mut self) {
        self.deref_mut().indent()
    }

    #[inline]
    fn dedent(&mut self) {
        self.deref_mut().dedent()
    }

    #[inline]
    fn set_last(&mut self) {
        self.deref_mut().set_last()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.deref().is_empty()
    }
}
