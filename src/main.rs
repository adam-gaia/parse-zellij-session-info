use anyhow::Result;
use jiff::Span;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::str::FromStr;
use winnow::Parser;
use winnow::ascii::dec_uint;
use winnow::ascii::space1;
use winnow::combinator::alt;
use winnow::combinator::delimited;
use winnow::combinator::eof;
use winnow::combinator::opt;
use winnow::combinator::seq;
use winnow::token::take_until;
use winnow_parse_error::ParseError;

#[derive(Debug, Serialize, Deserialize)]
enum State {
    Attached,
    Exited,
    Current,
}

impl Default for State {
    fn default() -> Self {
        Self::Attached
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Ago {
    years: Option<usize>,
    months: Option<usize>,
    days: Option<usize>,
    hours: Option<usize>,
    minutes: Option<usize>,
    seconds: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    name: String,
    created: Ago,
    state: State,
}

fn name(s: &mut &str) -> winnow::Result<String> {
    take_until(1.., " ").map(String::from).parse_next(s)
}

fn years(s: &mut &str) -> winnow::Result<usize> {
    let years = dec_uint.parse_next(s)?;
    let _ = alt(("years", "year")).parse_next(s)?;
    let _ = space.parse_next(s)?;
    Ok(years)
}

fn months(s: &mut &str) -> winnow::Result<usize> {
    let months = dec_uint.parse_next(s)?;
    let _ = alt(("months", "month")).parse_next(s)?;
    let _ = space.parse_next(s)?;
    Ok(months)
}

fn days(s: &mut &str) -> winnow::Result<usize> {
    let days = dec_uint.parse_next(s)?;
    let _ = alt(("days", "day")).parse_next(s)?;
    let _ = space.parse_next(s)?;
    Ok(days)
}

fn hours(s: &mut &str) -> winnow::Result<usize> {
    let hours = dec_uint.parse_next(s)?;
    let _ = "h".parse_next(s)?;
    let _ = space.parse_next(s)?;
    Ok(hours)
}

fn minutes(s: &mut &str) -> winnow::Result<usize> {
    let mins = dec_uint.parse_next(s)?;
    let _ = "m".parse_next(s)?;
    let _ = space.parse_next(s)?;
    Ok(mins)
}

fn seconds(s: &mut &str) -> winnow::Result<usize> {
    let secs = dec_uint.parse_next(s)?;
    let _ = "s".parse_next(s)?;
    let _ = space.parse_next(s)?;
    Ok(secs)
}

fn ago(s: &mut &str) -> winnow::Result<Ago> {
    let _ = "Created ".parse_next(s)?;
    let years = opt(years).parse_next(s)?;
    let months = opt(months).parse_next(s)?;
    let days = opt(days).parse_next(s)?;
    let hours = opt(hours).parse_next(s)?;
    let minutes = opt(minutes).parse_next(s)?;
    let seconds = opt(seconds).parse_next(s)?;
    let _ = "ago".parse_next(s)?;
    Ok(Ago {
        years,
        months,
        days,
        hours,
        minutes,
        seconds,
    })
}

fn created(s: &mut &str) -> winnow::Result<Ago> {
    delimited('[', ago, ']').parse_next(s)
}

fn current(s: &mut &str) -> winnow::Result<State> {
    "current".map(|_| State::Current).parse_next(s)
}

fn exited(s: &mut &str) -> winnow::Result<State> {
    "EXITED - attach to resurrect"
        .map(|_| State::Exited)
        .parse_next(s)
}

fn attached(s: &mut &str) -> winnow::Result<State> {
    let _ = space1.parse_next(s)?;
    Ok(State::Attached)
}

fn _state(s: &mut &str) -> winnow::Result<State> {
    alt((delimited('(', alt((current, exited)), ')'), attached)).parse_next(s)
}

fn state_or_default(s: &mut &str) -> winnow::Result<State> {
    match opt(alt((eof, space1))).parse_next(s)? {
        Some(_) => Ok(State::default()),
        None => state.parse_next(s),
    }
}

fn state(s: &mut &str) -> winnow::Result<State> {
    let eof = opt(eof).parse_next(s)?;
    match eof {
        Some(_) => Ok(State::default()),
        None => alt((
            "(current)".map(|_| State::Current),
            "(EXITED - attach to resurrect)".map(|_| State::Exited),
        ))
        .parse_next(s),
    }
}

fn space(s: &mut &str) -> winnow::Result<()> {
    let _ = " ".parse_next(s)?;
    Ok(())
}

fn session(s: &mut &str) -> winnow::Result<Session> {
    seq! {Session{
        name: name,
        _: space,
        created: created,
        _: space,
        state: state
    }}
    .parse_next(s)
}

impl FromStr for Session {
    type Err = ParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        session.parse(s).map_err(|e| ParseError::from_parse(e))
    }
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut sessions = Vec::new();
    for line in input.lines() {
        let session = Session::from_str(&line)?;
        sessions.push(session);
    }

    let repr = serde_json::to_string_pretty(&sessions)?;
    println!("{repr}");

    Ok(())
}
