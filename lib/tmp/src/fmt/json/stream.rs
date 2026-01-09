//! JSON Streams

use core::ops::ControlFlow as Flow;

use crate::{
    fmt::json::{
        self,
        token::Token,
    },
    io,
};

pub struct Null {
}

pub struct Bool {
    pub v: bool,
}

pub struct Number<'data> {
    pub v: &'data str,
}

pub struct String<'data> {
    pub v: &'data str,
}

pub enum Error<'data> {
    Foobar,
    Foobar2(&'data str),
}

pub struct Prim<'data> {
    pub v: &'data str,
}

pub enum Item<'data> {
    Error(Error<'data>),
    Prim,
    Key,
    ArrayOpen,
    ArrayClose,
    ObjectOpen,
    ObjectClose,
}

#[derive(Clone, Copy)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum State {
    Root,
    RootDone,
    ArrayOpen,
    ArrayValue,
    ArrayComma,
    ObjectOpen,
    ObjectKey,
    ObjectColon,
    ObjectValue,
    ObjectComma,
}

#[derive(Clone, Copy)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum Stack {
    Array,
    Object,
}

pub struct DecInner {
    state: State,
    stack: alloc::vec::Vec<Stack>,
}

pub struct Dec<'read> {
    inner: DecInner,
    tokenizer: json::token::Dec<'read>,
}

impl DecInner {
    pub fn new() -> Self {
        Self {
            state: State::Root,
            stack: alloc::vec::Vec::new(),
        }
    }

    fn unexpected(&mut self) -> Item<'static> {
        Item::Error(Error::Foobar)
    }

    fn propagate<'token>(&mut self, _error: json::token::Error<'token>) -> Item<'token> {
        Item::Error(Error::Foobar)
    }

    fn value<'token>(
        &mut self,
        _token: Token<'token>,
    ) -> Item<'token> {
        // XXX:
        Item::Prim
    }

    fn array_open(&mut self, from: Option<Stack>) -> Item<'static> {
        if let Some(v) = from {
            self.stack.push(v);
        }
        self.state = State::ArrayOpen;
        Item::ArrayOpen
    }

    fn array_close(&mut self) -> Item<'static> {
        self.state = match self.stack.pop() {
            None => State::RootDone,
            Some(Stack::Array) => State::ArrayValue,
            Some(Stack::Object) => State::ObjectValue,
        };
        Item::ArrayClose
    }

    fn object_open(&mut self, from: Option<Stack>) -> Item<'static> {
        if let Some(v) = from {
            self.stack.push(v);
        }
        self.state = State::ObjectOpen;
        Item::ObjectOpen
    }

    fn object_close(&mut self) -> Item<'static> {
        self.state = match self.stack.pop() {
            None => State::RootDone,
            Some(Stack::Array) => State::ArrayValue,
            Some(Stack::Object) => State::ObjectValue,
        };
        Item::ObjectClose
    }

    fn advance<'token>(
        &mut self,
        token: Token<'token>,
    ) -> Flow<io::stream::More, Option<Item<'token>>> {
        let item: Option<Item<'token>> = match (self.state, token) {
            (State::Root, Token::Error(e)) => Some(self.propagate(e)),
            (State::Root, Token::Whitespace { .. }) => None,
            (State::Root, Token::Colon) => Some(self.unexpected()),
            (State::Root, Token::Comma) => Some(self.unexpected()),
            (State::Root, Token::ArrayOpen) => Some(self.array_open(None)),
            (State::Root, Token::ArrayClose) => Some(self.unexpected()),
            (State::Root, Token::ObjectOpen) => Some(self.object_open(None)),
            (State::Root, Token::ObjectClose) => Some(self.unexpected()),
            (State::Root, Token::Null) => Some(self.value(token)),
            (State::Root, Token::False) => Some(self.value(token)),
            (State::Root, Token::True) => Some(self.value(token)),
            (State::Root, Token::Number { .. }) => Some(self.value(token)),
            (State::Root, Token::String { .. }) => Some(self.value(token)),

            (State::ArrayOpen, Token::Error(e)) => Some(self.propagate(e)),
            (State::ArrayOpen, Token::Whitespace { .. }) => None,
            (State::ArrayOpen, Token::Colon) => Some(self.unexpected()),
            (State::ArrayOpen, Token::Comma) => Some(self.unexpected()),
            (State::ArrayOpen, Token::ArrayOpen) => Some(self.array_open(Some(Stack::Array))),
            (State::ArrayOpen, Token::ArrayClose) => Some(self.array_close()),
            (State::ArrayOpen, Token::ObjectOpen) => Some(self.object_open(Some(Stack::Array))),
            (State::ArrayOpen, Token::ObjectClose) => Some(self.unexpected()),
            (State::ArrayOpen, Token::Null) => Some(self.value(token)),
            (State::ArrayOpen, Token::False) => Some(self.value(token)),
            (State::ArrayOpen, Token::True) => Some(self.value(token)),
            (State::ArrayOpen, Token::Number { .. }) => Some(self.value(token)),
            (State::ArrayOpen, Token::String { .. }) => Some(self.value(token)),

            (State::ArrayValue, Token::Error(e)) => Some(self.propagate(e)),
            (State::ArrayValue, Token::Whitespace { .. }) => None,
            (State::ArrayValue, Token::Colon) => Some(self.unexpected()),
            (State::ArrayValue, Token::Comma) => { self.state = State::ArrayComma; None },
            (State::ArrayValue, Token::ArrayOpen) => Some(self.unexpected()),
            (State::ArrayValue, Token::ArrayClose) => Some(self.array_close()),
            (State::ArrayValue, Token::ObjectOpen) => Some(self.unexpected()),
            (State::ArrayValue, Token::ObjectClose) => Some(self.unexpected()),
            (State::ArrayValue, Token::Null) => Some(self.unexpected()),
            (State::ArrayValue, Token::False) => Some(self.unexpected()),
            (State::ArrayValue, Token::True) => Some(self.unexpected()),
            (State::ArrayValue, Token::Number { .. }) => Some(self.unexpected()),
            (State::ArrayValue, Token::String { .. }) => Some(self.unexpected()),

            (State::ArrayComma, Token::Error(e)) => Some(self.propagate(e)),
            (State::ArrayComma, Token::Whitespace { .. }) => None,
            (State::ArrayComma, Token::Colon) => Some(self.unexpected()),
            (State::ArrayComma, Token::Comma) => Some(self.unexpected()),
            (State::ArrayComma, Token::ArrayOpen) => Some(self.array_open(Some(Stack::Array))),
            (State::ArrayComma, Token::ArrayClose) => Some(self.unexpected()),
            (State::ArrayComma, Token::ObjectOpen) => Some(self.object_open(Some(Stack::Array))),
            (State::ArrayComma, Token::ObjectClose) => Some(self.unexpected()),
            (State::ArrayComma, Token::Null) => Some(self.value(token)),
            (State::ArrayComma, Token::False) => Some(self.value(token)),
            (State::ArrayComma, Token::True) => Some(self.value(token)),
            (State::ArrayComma, Token::Number { .. }) => Some(self.value(token)),
            (State::ArrayComma, Token::String { .. }) => Some(self.value(token)),

            (State::ObjectOpen, Token::Error(e)) => Some(self.propagate(e)),
            (State::ObjectOpen, Token::Whitespace { .. }) => None,
            (State::ObjectOpen, Token::Colon) => Some(self.unexpected()),
            (State::ObjectOpen, Token::Comma) => Some(self.unexpected()),
            (State::ObjectOpen, Token::ArrayOpen) => Some(self.unexpected()),
            (State::ObjectOpen, Token::ArrayClose) => Some(self.unexpected()),
            (State::ObjectOpen, Token::ObjectOpen) => Some(self.unexpected()),
            (State::ObjectOpen, Token::ObjectClose) => Some(self.unexpected()),
            (State::ObjectOpen, Token::Null) => Some(self.unexpected()),
            (State::ObjectOpen, Token::False) => Some(self.unexpected()),
            (State::ObjectOpen, Token::True) => Some(self.unexpected()),
            (State::ObjectOpen, Token::Number { .. }) => Some(self.unexpected()),
            (State::ObjectOpen, Token::String { .. }) => { self.state = State::ObjectKey; Some(self.value(token)) },

            (State::ObjectKey, Token::Error(e)) => Some(self.propagate(e)),
            (State::ObjectKey, Token::Whitespace { .. }) => None,
            (State::ObjectKey, Token::Colon) => { self.state = State::ObjectColon; None },
            (State::ObjectKey, Token::Comma) => Some(self.unexpected()),
            (State::ObjectKey, Token::ArrayOpen) => Some(self.unexpected()),
            (State::ObjectKey, Token::ArrayClose) => Some(self.unexpected()),
            (State::ObjectKey, Token::ObjectOpen) => Some(self.unexpected()),
            (State::ObjectKey, Token::ObjectClose) => Some(self.unexpected()),
            (State::ObjectKey, Token::Null) => Some(self.unexpected()),
            (State::ObjectKey, Token::False) => Some(self.unexpected()),
            (State::ObjectKey, Token::True) => Some(self.unexpected()),
            (State::ObjectKey, Token::Number { .. }) => Some(self.unexpected()),
            (State::ObjectKey, Token::String { .. }) => Some(self.unexpected()),

            (State::ObjectColon, Token::Error(e)) => Some(self.propagate(e)),
            (State::ObjectColon, Token::Whitespace { .. }) => None,
            (State::ObjectColon, Token::Colon) => Some(self.unexpected()),
            (State::ObjectColon, Token::Comma) => Some(self.unexpected()),
            (State::ObjectColon, Token::ArrayOpen) => Some(self.array_open(Some(Stack::Object))),
            (State::ObjectColon, Token::ArrayClose) => Some(self.unexpected()),
            (State::ObjectColon, Token::ObjectOpen) => Some(self.object_open(Some(Stack::Object))),
            (State::ObjectColon, Token::ObjectClose) => Some(self.unexpected()),
            (State::ObjectColon, Token::Null) => Some(self.value(token)),
            (State::ObjectColon, Token::False) => Some(self.value(token)),
            (State::ObjectColon, Token::True) => Some(self.value(token)),
            (State::ObjectColon, Token::Number { .. }) => Some(self.value(token)),
            (State::ObjectColon, Token::String { .. }) => Some(self.value(token)),

            (State::ObjectValue, Token::Error(e)) => Some(self.propagate(e)),
            (State::ObjectValue, Token::Whitespace { .. }) => None,
            (State::ObjectValue, Token::Colon) => Some(self.unexpected()),
            (State::ObjectValue, Token::Comma) => { self.state = State::ObjectComma; None },
            (State::ObjectValue, Token::ArrayOpen) => Some(self.unexpected()),
            (State::ObjectValue, Token::ArrayClose) => Some(self.unexpected()),
            (State::ObjectValue, Token::ObjectOpen) => Some(self.unexpected()),
            (State::ObjectValue, Token::ObjectClose) => Some(self.object_close()),
            (State::ObjectValue, Token::Null) => Some(self.unexpected()),
            (State::ObjectValue, Token::False) => Some(self.unexpected()),
            (State::ObjectValue, Token::True) => Some(self.unexpected()),
            (State::ObjectValue, Token::Number { .. }) => Some(self.unexpected()),
            (State::ObjectValue, Token::String { .. }) => Some(self.unexpected()),

            (State::ObjectComma, Token::Error(e)) => Some(self.propagate(e)),
            (State::ObjectComma, Token::Whitespace { .. }) => None,
            (State::ObjectComma, Token::Colon) => Some(self.unexpected()),
            (State::ObjectComma, Token::Comma) => Some(self.unexpected()),
            (State::ObjectComma, Token::ArrayOpen) => Some(self.unexpected()),
            (State::ObjectComma, Token::ArrayClose) => Some(self.unexpected()),
            (State::ObjectComma, Token::ObjectOpen) => Some(self.unexpected()),
            (State::ObjectComma, Token::ObjectClose) => Some(self.unexpected()),
            (State::ObjectComma, Token::Null) => Some(self.unexpected()),
            (State::ObjectComma, Token::False) => Some(self.unexpected()),
            (State::ObjectComma, Token::True) => Some(self.unexpected()),
            (State::ObjectComma, Token::Number { .. }) => Some(self.unexpected()),
            (State::ObjectComma, Token::String { .. }) => { self.state = State::ObjectKey; Some(self.value(token)) },

            (State::RootDone, Token::Error(e)) => Some(self.propagate(e)),
            (State::RootDone, Token::Whitespace { .. }) => None,
            (State::RootDone, Token::Colon) => Some(self.unexpected()),
            (State::RootDone, Token::Comma) => Some(self.unexpected()),
            (State::RootDone, Token::ArrayOpen) => Some(self.unexpected()),
            (State::RootDone, Token::ArrayClose) => Some(self.unexpected()),
            (State::RootDone, Token::ObjectOpen) => Some(self.unexpected()),
            (State::RootDone, Token::ObjectClose) => Some(self.unexpected()),
            (State::RootDone, Token::Null) => Some(self.unexpected()),
            (State::RootDone, Token::False) => Some(self.unexpected()),
            (State::RootDone, Token::True) => Some(self.unexpected()),
            (State::RootDone, Token::Number { .. }) => Some(self.unexpected()),
            (State::RootDone, Token::String { .. }) => Some(self.unexpected()),
        };

        Flow::Continue(item)
    }
}

impl<'read> Dec<'read> {
    pub fn with(
        read: &'read mut dyn io::stream::Read,
    ) -> Self {
        Self {
            inner: DecInner::new(),
            tokenizer: json::token::Dec::with(read),
        }
    }

    fn advance_inner(&mut self) -> Flow<io::stream::More, Option<Item<'_>>> {
        let token = self.tokenizer.pop()?;
        self.inner.advance(token)
    }

    fn advance<'this>(&'this mut self) -> Flow<io::stream::More, Item<'this>> {
        // Keep this function as minimal as possible. Put all logic into
        // `Self::advance_inner()` to ensure we do not accidentally break
        // lifetime annotations.
        //
        // This split is only needed to work around an NLL limitation, which
        // cannot correctly limit mutable borrows in a loop if the borrow is
        // an unconditional return.
        //
        // This function compiles fine with Polonius as borrow checker, so
        // hopefully this is a temporary workaround.
        loop {
            match self.advance_inner()? {
                None => {},
                Some(v) => {
                    return Flow::Continue(
                        // SAFETY: Workaround for NLL, unneeded with Polonius.
                        unsafe {
                            core::mem::transmute::<
                                Item<'_>,
                                Item<'this>,
                            >(v)
                        },
                    );
                }
            }
        }
    }

    pub fn pop(&mut self) -> Flow<io::stream::More, Item<'_>> {
        self.advance()
    }
}
