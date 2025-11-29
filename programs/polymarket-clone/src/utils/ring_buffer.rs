use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_EVENTS, MAX_REQUESTS},
    error::MarketError,
    state::{Event, EventQueue, Request, RequestQueue},
};

pub fn enqueue_request(queue: &mut RequestQueue, request: Request) -> Result<()> {
    if (queue.count as usize) >= MAX_REQUESTS {
        return err!(MarketError::RequestQueueFull); //this is a macro given by anchor_lang error module
    }

    let index = ((queue.count + queue.head) as usize) % MAX_REQUESTS;
    queue.requests[index] = request;
    queue.count = queue.count.checked_add(1).ok_or(MarketError::MathError)?;

    Ok(())
}

pub fn dequeue_requests(queue: &mut RequestQueue, n: usize) -> Result<Vec<Request>> {
    let mut output_queue: Vec<Request> = Vec::new();
    let available_requests = queue.count as usize;
    let traversal_count = core::cmp::min(n, available_requests);
    for i in 0..traversal_count {
        let index = ((queue.head as usize) + i) % MAX_REQUESTS;
        let request = queue.requests[index];
        output_queue.push(request);
    }

    queue.head = queue
        .head
        .checked_add(traversal_count as u64)
        .ok_or(MarketError::MathError)?;
    queue.count = queue
        .count
        .checked_sub(traversal_count as u64)
        .ok_or(MarketError::MathError)?;

    Ok(output_queue)
}

pub fn push_event(queue: &mut EventQueue, event: Event) -> Result<()> {
    if (queue.count as usize) >= MAX_EVENTS {
        return err!(MarketError::MathError);
    }

    let index = ((queue.head + queue.count) as usize) % MAX_EVENTS;
    queue.events[index] = event;
    queue.count = queue.count.checked_add(1).ok_or(MarketError::MathError)?;

    Ok(())
}

pub fn peek_events(queue: &EventQueue, n: usize) -> Vec<Event> {
    //why not result because this code has mo chance of failing thats just returning the value without handling Capacitythe errors
    let available_events = queue.count as usize;
    let read_count = core::cmp::min(n, available_events);
    let mut read_event = Vec::new();
    for i in 0..read_count {
        let index = ((queue.head as usize) + i) % MAX_EVENTS;
        read_event.push(queue.events[index]);
    }
    read_event
}

pub fn pop_events(queue: &mut EventQueue, n: usize) -> Result<Vec<Event>> {
    let events = peek_events(queue, n);
    queue.head = queue
        .head
        .checked_add(events.len() as u64)
        .ok_or(MarketError::MathError)?;
    queue.count = queue
        .count
        .checked_sub(events.len() as u64)
        .ok_or(MarketError::MathError)?;
    Ok(events)
}
