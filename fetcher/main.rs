#![feature(conservative_impl_trait)]

extern crate common;
#[macro_use]
extern crate log;
extern crate futures;
extern crate tokio_core;
extern crate tokio_request;
extern crate rss;
extern crate time;
extern crate mailparse;
extern crate uuid;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate readability;

use std::cmp;
use std::str;
use std::ascii::AsciiExt;
use std::net::SocketAddr;
use std::io::{Result as IoResult, Error as IoError, ErrorKind as IoErrorKind};

use time::Timespec;
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::{UdpSocket, UdpCodec};
use futures::future;
use futures::{Future, Stream};
use rss::Channel;
use uuid::{Uuid, NAMESPACE_X500};
use diesel::prelude::*;
use diesel::pg::PgConnection;

use common::logger;
use common::schema;
use common::types::{Url, Key};
use models::{Feed, NewEntry};
use scheduler::Scheduler;
use readability::Readability;

mod scheduler;
mod download;
mod models;

const MIN_INTERVAL: u32 = 3600;
const MAX_INTERVAL: u32 = 24 * 3600;
const PROMPTNESS: f32 = 0.5;

fn estimate_interval(prev: u32, total: u32, new: u32) -> u32 {
    if total == 0 {
        return cmp::min(prev + MIN_INTERVAL, MAX_INTERVAL);
    }

    let staleness = (total - new) as f32 / total as f32;
    let delta = staleness - PROMPTNESS;
    let estimated = prev as f32 * (1. + delta / PROMPTNESS);
    let trust = (total as f32 / 30.).min(1.);

    let next = prev as f32 * (1. - trust) + estimated * trust;

    cmp::max(MIN_INTERVAL, cmp::min(next as u32, MAX_INTERVAL))
}

fn purify_text(string: String) -> Option<String> {
    if !string.is_empty() && string.trim().len() == string.len() {
        return Some(string);
    }

    let string = string.trim();

    if string.is_empty() {
        None
    } else {
        Some(string.to_owned())
    }
}

fn unify_language(mut language: String) -> Option<String> {
    if !language.is_ascii() {
        warn!("Cannot convert \"{}\" to a language code: not ascii", language);
        return None;
    }

    if language.len() == 2 {
        language.make_ascii_lowercase();
        Some(language)
    } else if language.len() > 2 {
        Some(language.trim()[..2].to_lowercase())
    } else {
        warn!("Cannot convert \"{}\" to a language code", language);
        None
    }
}

fn parse_rfc822_date(date: &str) -> Option<Timespec> {
    match mailparse::dateparse(date) {
        Ok(date) => Some(Timespec::new(date, 0)),
        Err(error) => {
            warn!("Cannot parse \"{}\" as date: {}", date, error);
            None
        }
    }
}

fn parse_url(url: &str) -> Option<Url> {
    match Url::parse(url.trim()) {
        Ok(url) => Some(url),
        Err(_) => {
            warn!("Cannot parse \"{}\" as url", url);
            None
        }
    }
}

fn fetch_entries(handle: &Handle, mut feed: Feed)
    -> impl Future<Item=(Feed, Vec<NewEntry>), Error=()>
{
    info!("Fetching {} feed...", feed.key);

    download::channel(handle, &feed.url).then(|channel| {
        Ok(match channel {
            Ok(channel) => disassemble_channel(feed, channel),
            Err(error) => {
                warn!("Fetching {} is failed: {}", feed.key, error);

                let interval = estimate_interval(feed.interval.unwrap() as u32, 0, 0);
                feed.interval = Some(interval as i32);

                (feed, Vec::new())
            }
        })
    })
}

fn disassemble_channel(mut feed: Feed, channel: Channel) -> (Feed, Vec<NewEntry>) {
    // TODO(loyd): use `channel.ttl` as some assumption about `feed.interval`.
    feed.title = purify_text(channel.title);
    feed.description = purify_text(channel.description);
    feed.language = channel.language.and_then(unify_language);
    feed.logo = channel.image.and_then(|image| Url::parse(&image.url).ok());
    feed.copyright = channel.copyright.and_then(purify_text);

    let augmented = feed.augmented.unwrap_or(Timespec::new(0, 0));

    let (mut total_count, mut new_count) = (0, 0);

    let entries = channel.items.into_iter().filter_map(|item| {
        let published = item.pub_date.and_then(|date| parse_rfc822_date(&date));

        if let Some(published) = published {
            total_count += 1;

            if published <= augmented {
                return None;
            }

            new_count += 1;
        }

        let url = item.link.and_then(|url| parse_url(&url));

        let key = if let Some(ref url) = url {
            Key::from(url.clone())
        } else if let Some(ref title) = item.title {
            Key::from(Uuid::new_v5(&NAMESPACE_X500, title))
        } else {
            return None;
        };

        Some(NewEntry {
            key,
            url,
            published,
            feed_id: feed.id,
            title: item.title.and_then(purify_text),
            author: item.author.and_then(purify_text),
            description: item.description.and_then(purify_text),
            content: item.content.and_then(purify_text)
        })
    }).collect();

    let interval = estimate_interval(feed.interval.unwrap() as u32, total_count, new_count);
    feed.interval = Some(interval as i32);

    (feed, entries)
}

fn fetch_documents(handle: &Handle, feed: Feed, entries: Vec<NewEntry>)
    -> impl Future<Item=(Feed, Vec<NewEntry>), Error=()> + 'static
{
    let fetchers = entries.into_iter().map(|mut entry| {
        debug!("  Fetching {} entry...", entry.key);

        if entry.url.is_none() {
            return future::ok(Some(entry)).boxed();
        }

        let download = download::document(handle, entry.url.as_ref().unwrap());

        download.then(|result| {
            let document = match result {
                Ok(document) => document,
                Err(error) => {
                    warn!("Fetching {} is failed: {}", entry.key, error);
                    return Ok(None);
                }
            };

            // TODO(loyd): should we use a thread pool here?
            let content = Readability::new().parse(&document).to_string();

            // TODO(loyd): leave original `content` in some situations.
            entry.content = Some(content);
            Ok(Some(entry))
        }).boxed()
    }).collect::<Vec<_>>();

    future::join_all(fetchers).map(|entries| {
        let entries = entries.into_iter().filter_map(|entry| entry).collect();
        (feed, entries)
    })
}

fn save_entries(connection: &PgConnection, feed: &Feed, entries: &[NewEntry]) -> QueryResult<bool> {
    // TODO(loyd): should we use a connection pool here?
    connection.transaction(|| {
        let active = diesel::update(feed)
            .set(feed)
            .execute(connection)? > 0;

        diesel::insert(entries)
            .into(schema::entry::table)
            .execute(connection)?;

        Ok(active)
    })
}

struct IdCodec;

impl UdpCodec for IdCodec {
    type In = i32;
    type Out = ();

    fn decode(&mut self, _: &SocketAddr, buffer: &[u8]) -> IoResult<i32> {
        let message = str::from_utf8(buffer).map_err(|_| IoErrorKind::InvalidInput)?;

        let id = message
            .trim()
            .parse::<i32>()
            .map_err(|message| IoError::new(IoErrorKind::InvalidData, message))?;

        Ok(id)
    }

    fn encode(&mut self, _: (), _: &mut Vec<u8>) -> SocketAddr {
        unimplemented!();
    }
}

fn main() {
    logger::init().unwrap();

    let connection = schema::establish_connection().unwrap();

    let (scheduler, feed_stream) = Scheduler::new();

    info!("Loading feeds...");

    let initial_feeds = schema::feed::table.load::<Feed>(&connection).unwrap();

    info!("Scheduling initial {} feeds...", initial_feeds.len());

    for mut feed in initial_feeds {
        let timeout = feed.interval.unwrap_or(0);
        feed.interval = Some(timeout);

        debug!("  Scheduled {} after {}s", feed.key, timeout);

        scheduler.schedule((timeout * 1000) as u64, feed);
    }

    info!("Running the reactor...");

    let mut lp = Core::new().unwrap();
    let handle = lp.handle();

    let fetching = feed_stream
        // TODO(loyd): should we fetch feeds concurrently?
        .and_then(|feed| fetch_entries(&handle, feed))
        .and_then(|(feed, entries)| fetch_documents(&handle, feed, entries))
        .for_each(|(mut feed, entries)| {
            info!("Visited {} and collected {} new entries", feed.key, entries.len());

            if let Some(augmented) = entries.iter().filter_map(|entry| entry.published).max() {
                feed.augmented = Some(augmented);
            }

            if save_entries(&connection, &feed, &entries).unwrap() {
                debug!("  Scheduled after {}s", feed.interval.unwrap());

                scheduler.schedule((feed.interval.unwrap() * 1000) as u64, feed);
            }

            Ok(())
        });

    let addr = &"127.0.0.1:3001".parse().unwrap();

    let adding = UdpSocket::bind(addr, &handle).unwrap().framed(IdCodec)
        .map(|id| {
            match schema::feed::table.find(id).first::<Feed>(&connection) {
                Ok(mut feed) => {
                    debug!("Scheduled {} anytime soon", feed.key);
                    feed.interval = Some(0);
                    scheduler.schedule(0, feed);
                },
                Err(error) => error!("Loading feed #{} is failed: {}", id, error)
            };

            ()
        })
        .or_else(|error| {
            error!("Invalid id: {}", error);
            Ok(())
        })
        .for_each(|_| Ok(()));

    let _ = lp.run(fetching.select(adding));
}

#[test]
fn it_estimates_interval() {
    assert_eq!(estimate_interval(MIN_INTERVAL, 30, 0), (MIN_INTERVAL as f32 / PROMPTNESS) as u32);

    // Keypoints.
    let some_prev = MIN_INTERVAL + 2048;
    assert_eq!(estimate_interval(some_prev, 10000, ((1. - PROMPTNESS) * 10000.) as u32), some_prev);
    assert_eq!(estimate_interval(MIN_INTERVAL, 30, 30), MIN_INTERVAL);
    assert_eq!(estimate_interval(MIN_INTERVAL, 1, 1), MIN_INTERVAL);
    assert_eq!(estimate_interval(MAX_INTERVAL, 30, 0), MAX_INTERVAL);
    assert_eq!(estimate_interval(MAX_INTERVAL, 1, 0), MAX_INTERVAL);
    assert_eq!(estimate_interval(MIN_INTERVAL + 42, 0, 0), 2 * MIN_INTERVAL + 42);
    assert_eq!(estimate_interval(0, 30, 30), MIN_INTERVAL);
    assert_eq!(estimate_interval(0, 30, 0), MIN_INTERVAL);
    assert_eq!(estimate_interval(0, 1, 0), MIN_INTERVAL);
    assert_eq!(estimate_interval(0, 0, 0), MIN_INTERVAL);
}
