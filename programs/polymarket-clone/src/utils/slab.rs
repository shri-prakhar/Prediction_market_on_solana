use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_ORDER_ENTRIES, MAX_PRICE_NODES},
    error::MarketError,
    state::Slab,
};

pub fn initialize_slab(slab: &mut Slab, is_bid: bool, bump: u8) {
    slab.is_bid = is_bid;
    slab.node_count = 0;
    slab.free_order_entry_head = -1;
    slab.free_price_node_head = -1;
    slab.root_price_node = -1;
    slab.bump = bump;

    for i in 0..MAX_PRICE_NODES {
        slab.price_nodes[i].occupied = false;
        slab.price_nodes[i].key = 0;
        slab.price_nodes[i].order_head = -1;
        slab.price_nodes[i].order_tail = -1;
        slab.price_nodes[i].left = -1;
        slab.price_nodes[i].right = -1;
        slab.price_nodes[i].parent = -1;
        slab.price_nodes[i].color = 0;
    }
    for i in 0..MAX_ORDER_ENTRIES {
        slab.order_entries[i].occupied = false;
        slab.order_entries[i].next_in_price = -1;
        slab.order_entries[i].order_id = 0;
        slab.order_entries[i].owner_slot = 0;
        slab.order_entries[i].quantity = 0;
        slab.order_entries[i].reserved_amount = 0;
        slab.order_entries[i].open_order_owner = Pubkey::default();
    }
}

#[inline(always)]
fn is_null(index: i32) -> bool {
    if index == -1 {
        true
    } else {
        false
    }
}

#[inline(always)]
fn is_red(slab: &Slab, index: i32) -> bool {
    if is_null(index) {
        false
    } else {
        slab.price_nodes[index as usize].color == 1
    }
}

#[inline(always)]
fn set_color(slab: &mut Slab, index: i32, color: u8) {
    if !is_null(index) {
        slab.price_nodes[index as usize].color = color;
    }
}

pub fn allocate_price_node(slab: &mut Slab) -> Result<i32> {
    for i in 0..(MAX_PRICE_NODES as i32) {
        if !slab.price_nodes[i as usize].occupied {
            slab.price_nodes[i as usize].right = -1;
            slab.price_nodes[i as usize].parent = -1;
            slab.price_nodes[i as usize].order_tail = -1;
            slab.price_nodes[i as usize].order_head = -1;
            slab.price_nodes[i as usize].occupied = true;
            slab.price_nodes[i as usize].left = -1;
            slab.price_nodes[i as usize].color = 1;
            slab.node_count = slab
                .node_count
                .checked_add(1)
                .ok_or(MarketError::MathError)?;
            return Ok(i); // so in rust Ok() alone is just an expression it doesn't return the okay value to the user so return is necessary because it return this expression at the last pf the function when no further execution limit left
        }
    }
    Err(error!(MarketError::MathError))
}

pub fn allocate_order_entry(slab: &mut Slab) -> Result<i32> {
    for i in 0..(MAX_ORDER_ENTRIES as i32) {
        if !slab.order_entries[i as usize].occupied {
            slab.order_entries[i as usize].occupied = true;
            slab.order_entries[i as usize].next_in_price = -1;
            return Ok(i);
        }
    }
    Err(error!(MarketError::MathError))
}

pub fn find_price_node_index(slab: &Slab, price: u128) -> Option<i32> {
    let mut current_index = slab.root_price_node;
    while !is_null(current_index) {
        let current_price = slab.price_nodes[current_index as usize].key;
        if current_price == price {
            return Some(current_index);
        }
        if price < current_price {
            current_index = slab.price_nodes[current_index as usize].left;
        } else {
            current_index = slab.price_nodes[current_index as usize].right;
        }
    }
    None
}

fn left_rotate(slab: &mut Slab, x: i32) {
    let y = slab.price_nodes[x as usize].right;
    if is_null(y) {
        return;
    }

    slab.price_nodes[x as usize].right = slab.price_nodes[y as usize].left;
    if !is_null(slab.price_nodes[y as usize].left) {
        let l = slab.price_nodes[y as usize].left;
        slab.price_nodes[l as usize].parent = x;
    }

    slab.price_nodes[y as usize].parent = slab.price_nodes[x as usize].parent;
    if slab.root_price_node == x {
        slab.root_price_node = y;
    } else {
        let x_parent = slab.price_nodes[x as usize].parent;
        if x == slab.price_nodes[x_parent as usize].left {
            slab.price_nodes[x_parent as usize].left = y;
        } else {
            slab.price_nodes[x_parent as usize].right = y;
        }
    }

    slab.price_nodes[y as usize].left = x;
    slab.price_nodes[x as usize].parent = y;
}

fn right_rotate(slab: &mut Slab, x: i32) {
    let y = slab.price_nodes[x as usize].left;
    if is_null(y) {
        return;
    }
    slab.price_nodes[x as usize].left = slab.price_nodes[y as usize].right;
    if !is_null(slab.price_nodes[y as usize].right) {
        let r = slab.price_nodes[y as usize].right;
        slab.price_nodes[r as usize].parent = x;
    }

    slab.price_nodes[y as usize].parent = slab.price_nodes[x as usize].parent;
    if is_null(slab.price_nodes[x as usize].parent) {
        slab.root_price_node = y;
    } else {
        let x_parent = slab.price_nodes[x as usize].parent;
        if x == slab.price_nodes[x_parent as usize].left {
            slab.price_nodes[x_parent as usize].left = y;
        } else {
            slab.price_nodes[x_parent as usize].right = y;
        }
    }

    slab.price_nodes[y as usize].right = x;
    slab.price_nodes[x as usize].parent = y;
}

fn insert_fixup(slab: &mut Slab, mut z: i32) {
    while is_red(slab, slab.price_nodes[z as usize].parent) {
        let parent = slab.price_nodes[z as usize].parent;
        let grand_parent = slab.price_nodes[parent as usize].parent;
        if parent == slab.price_nodes[grand_parent as usize].left {
            let uncle = slab.price_nodes[grand_parent as usize].right;
            if is_red(slab, uncle) {
                set_color(slab, parent, 0);
                set_color(slab, uncle, 0);
                set_color(slab, grand_parent, 1);
                z = grand_parent;
            } else {
                if z == slab.price_nodes[parent as usize].right {
                    z = parent;
                    left_rotate(slab, z);
                }
                let parent2 = slab.price_nodes[z as usize].parent;
                let grand_parent2 = slab.price_nodes[parent2 as usize].parent;
                set_color(slab, parent2, 0);
                set_color(slab, grand_parent2, 1);
                right_rotate(slab, grand_parent2);
            }
        } else {
            let uncle = slab.price_nodes[grand_parent as usize].left;
            if is_red(slab, uncle) {
                set_color(slab, uncle, 0);
                set_color(slab, parent, 0);
                set_color(slab, grand_parent, 1);
                z = grand_parent
            } else {
                if z == slab.price_nodes[parent as usize].left {
                    z = parent;
                    right_rotate(slab, z);
                }
                let parent2 = slab.price_nodes[z as usize].parent;
                let grand_parent2 = slab.price_nodes[parent2 as usize].parent;
                set_color(slab, parent2, 0);
                set_color(slab, grand_parent2, 1);
                left_rotate(slab, grand_parent2);
            }
        }
        if slab.root_price_node == -1 {
            break;
        }
    }
    set_color(slab, slab.root_price_node, 0);
}

pub fn insert_price_node_by_tree(slab: &mut Slab, price: u128) -> Result<i32> {
    if let Some(index) = find_price_node_index(slab, price) {
        return Ok(index);
    }
    let z = allocate_price_node(slab)?;
    slab.price_nodes[z as usize].key = price;
    let mut y = -1;
    let mut x = slab.root_price_node;
    while !is_null(x) {
        y = x;
        if price < slab.price_nodes[x as usize].key {
            x = slab.price_nodes[x as usize].left;
        } else {
            x = slab.price_nodes[x as usize].right;
        }
    }

    slab.price_nodes[z as usize].parent = y;

    if is_null(y) {
        slab.root_price_node = z;
    } else if price < slab.price_nodes[y as usize].key {
        slab.price_nodes[y as usize].left = z;
    } else {
        slab.price_nodes[y as usize].right = z;
    }

    slab.price_nodes[z as usize].color = 1;

    insert_fixup(slab, z);
    Ok(z)
}

fn transplant(slab: &mut Slab, u: i32, v: i32) {
    let u_parent = slab.price_nodes[u as usize].parent;
    if is_null(u_parent) {
        slab.root_price_node = v;
    } else if u == slab.price_nodes[u_parent as usize].left {
        slab.price_nodes[u_parent as usize].left = v;
    } else {
        slab.price_nodes[u_parent as usize].right = v;
    }
    if !is_null(v) {
        slab.price_nodes[v as usize].parent = u_parent;
    }
}

fn tree_min(slab: &mut Slab, mut x: i32) -> i32 {
    while !is_null(slab.price_nodes[x as usize].left) {
        x = slab.price_nodes[x as usize].left;
    }
    x
}

fn delete_fixup(slab: &mut Slab, mut x: i32, mut x_parent: i32) {
    while x != slab.root_price_node && (!(!is_null(x) && is_red(slab, x))) {
        if x_parent == -1 {
            break;
        }
        if x == slab.price_nodes[x_parent as usize].left {
            let mut sibling = slab.price_nodes[x_parent as usize].right;
            if is_red(slab, sibling) {
                set_color(slab, sibling, 0);
                set_color(slab, x_parent, 1);
                left_rotate(slab, x_parent);
                sibling = slab.price_nodes[x_parent as usize].right;
            }
            if !is_null(slab.price_nodes[sibling as usize].left)
                && !is_red(slab, slab.price_nodes[sibling as usize].left)
                && !is_null(slab.price_nodes[sibling as usize].right)
                && !is_red(slab, slab.price_nodes[sibling as usize].right)
            {
                set_color(slab, sibling, 1);
                x = x_parent;
                x_parent = slab.price_nodes[x as usize].parent;
            } else {
                if !is_null(slab.price_nodes[sibling as usize].right)
                    && !is_red(slab, slab.price_nodes[sibling as usize].right)
                {
                    let sibling_left = slab.price_nodes[sibling as usize].left;
                    set_color(slab, sibling_left, 0);
                    set_color(slab, sibling, 1);
                    right_rotate(slab, sibling);
                    sibling = slab.price_nodes[x_parent as usize].right;
                }
                set_color(slab, sibling, slab.price_nodes[x_parent as usize].color);
                set_color(slab, x_parent, 0);
                let sibling_right = slab.price_nodes[sibling as usize].right;
                set_color(slab, sibling_right, 0);
                left_rotate(slab, x_parent);
                x = slab.root_price_node;
                break;
            }
        } else {
            let mut sibling = slab.price_nodes[x_parent as usize].left;
            if is_red(slab, sibling) {
                set_color(slab, sibling, 0);
                set_color(slab, x_parent, 1);
                right_rotate(slab, x_parent);
                sibling = slab.price_nodes[x_parent as usize].left;
            }
            if !is_null(slab.price_nodes[sibling as usize].right)
                && !is_red(slab, slab.price_nodes[sibling as usize].right)
                && !is_null(slab.price_nodes[sibling as usize].left)
                && !is_red(slab, slab.price_nodes[sibling as usize].left)
            {
                set_color(slab, sibling, 1);
                x = x_parent;
                x_parent = slab.price_nodes[x as usize].parent
            } else {
                if !is_null(slab.price_nodes[sibling as usize].left)
                    && !is_red(slab, slab.price_nodes[sibling as usize].left)
                {
                    let sibling_right = slab.price_nodes[sibling as usize].right;
                    set_color(slab, sibling_right, 0);
                    set_color(slab, sibling, 1);
                    left_rotate(slab, sibling);
                    sibling = slab.price_nodes[x_parent as usize].left;
                }
                set_color(slab, sibling, slab.price_nodes[x_parent as usize].color);
                set_color(slab, x_parent, 0);
                let sibling_left = slab.price_nodes[sibling as usize].left;
                set_color(slab, sibling_left, 0);
                right_rotate(slab, x_parent);
                x = slab.root_price_node;
                break;
            }
        }
    }
    if !is_null(x) {
        set_color(slab, x, 0);
    }
}

pub fn remove_price_node(slab: &mut Slab, z: i32) -> Result<()> {
    if is_null(z) {
        return Ok(());
    }

    let mut y = z;
    let mut y_original_color = slab.price_nodes[y as usize].color;
    let x: i32;
    let x_parent: i32;

    if is_null(slab.price_nodes[z as usize].left) {
        x = slab.price_nodes[z as usize].right;
        x_parent = slab.price_nodes[z as usize].parent;
        transplant(slab, z, x);
    } else if is_null(slab.price_nodes[z as usize].right) {
        x = slab.price_nodes[z as usize].left;
        x_parent = slab.price_nodes[z as usize].parent;
        transplant(slab, z, x);
    } else {
        y = tree_min(slab, slab.price_nodes[z as usize].right);
        let y_color_before = slab.price_nodes[y as usize].color;
        y_original_color = y_color_before;
        x = slab.price_nodes[y as usize].right;
        if slab.price_nodes[y as usize].parent == z {
            if !is_null(x) {
                slab.price_nodes[x as usize].parent = y;
            }
            x_parent = y;
        } else {
            x_parent = slab.price_nodes[y as usize].parent;
            transplant(slab, y, slab.price_nodes[y as usize].right);
            slab.price_nodes[y as usize].right = slab.price_nodes[z as usize].right;
            if !is_null(slab.price_nodes[y as usize].right) {
                let right = slab.price_nodes[y as usize].right;
                slab.price_nodes[right as usize].parent = y;
            }
        }
        transplant(slab, z, y);
        slab.price_nodes[y as usize].left = slab.price_nodes[z as usize].left;
        if !is_null(slab.price_nodes[y as usize].left) {
            let left = slab.price_nodes[y as usize].left;
            slab.price_nodes[left as usize].parent = y;
        }
        slab.price_nodes[y as usize].color = slab.price_nodes[z as usize].color;
    }

    slab.price_nodes[z as usize].occupied = false;
    slab.price_nodes[z as usize].left = -1;
    slab.price_nodes[z as usize].right = -1;
    slab.price_nodes[z as usize].order_head = -1;
    slab.price_nodes[z as usize].order_tail = -1;
    slab.price_nodes[z as usize].key = 0;
    slab.price_nodes[z as usize].color = 0;
    slab.price_nodes[z as usize].parent = -1;
    slab.node_count = slab
        .node_count
        .checked_sub(1)
        .ok_or(MarketError::MathError)?;

    if y_original_color == 0 {
        delete_fixup(slab, x, x_parent);
    }

    Ok(())
}

pub fn append_order_to_price(
    slab: &mut Slab,
    price_node_index: i32,
    order_index: i32,
) -> Result<()> {
    if price_node_index < 0 {
        return err!(MarketError::MathError);
    }

    let price_node = &mut slab.price_nodes[price_node_index as usize];
    if price_node.order_head == -1 {
        price_node.order_head = order_index;
        price_node.order_tail = order_index;
    } else {
        let tail = price_node.order_tail;
        slab.order_entries[tail as usize].next_in_price = order_index;
        price_node.order_tail = order_index
    }
    slab.order_entries[order_index as usize].next_in_price = -1;
    Ok(())
}

pub fn pop_order_from_prices(slab: &mut Slab, price_node_index: i32) -> Result<i32> {
    if price_node_index < 0 {
        return err!(MarketError::NoMatchingOrder);
    }
    let price_node = &mut slab.price_nodes[price_node_index as usize];
    if price_node.order_head == -1 {
        return err!(MarketError::NoMatchingOrder);
    }
    let head = price_node.order_head;
    let next = slab.order_entries[head as usize].next_in_price;
    price_node.order_head = next;
    if price_node.order_head == -1 {
        price_node.order_tail = -1;
    }

    slab.order_entries[head as usize].occupied = false;
    slab.order_entries[head as usize].next_in_price = -1;
    Ok(head)
}

pub fn find_best_price_node_index(slab: &Slab) -> Option<i32> {
    let root = slab.root_price_node;
    if root == -1 {
        return None;
    }

    if slab.is_bid {
        // For bids, we want the highest price (rightmost node)
        let mut current = root;
        while slab.price_nodes[current as usize].right != -1 {
            current = slab.price_nodes[current as usize].right;
        }
        Some(current)
    } else {
        // For asks, we want the lowest price (leftmost node)
        let mut current = root;
        while slab.price_nodes[current as usize].left != -1 {
            current = slab.price_nodes[current as usize].left;
        }
        Some(current)
    }
}
