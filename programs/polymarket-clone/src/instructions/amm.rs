use crate::{constants::PRICE_PRECISION_SCALE, error::MarketError, state::OutcomeSide};
use anchor_lang::prelude::*;

pub const FP_SCALE: u128 = PRICE_PRECISION_SCALE;

pub fn amm_cost_to_yes(q_yes: u128, q_no: u128, b: u128, delta_q: u128) -> Result<u128> {
    let mut c1 = lsmr_cost(
        q_yes.checked_add(delta_q).ok_or(MarketError::MathError)?,
        q_no,
        b,
    )?;
    let c0 = lsmr_cost(q_yes, q_no, b)?;
    c1 = c1.checked_sub(c0).ok_or(MarketError::MathError)?;
    Ok(c1)
}

pub fn amm_cost_to_no(q_yes: u128, q_no: u128, b: u128, delta_q: u128) -> Result<u128> {
    let mut c1 = lsmr_cost(
        q_yes,
        q_no.checked_add(delta_q).ok_or(MarketError::MathError)?,
        b,
    )?;
    let c0 = lsmr_cost(q_yes, q_no, b)?;
    c1 = c1.checked_sub(c0).ok_or(MarketError::MathError)?;
    Ok(c1)
}

pub fn lsmr_cost(q_yes: u128, q_no: u128, b: u128) -> Result<u128> {
    if b == 0 {
        return err!(MarketError::InvalidArgument);
    }

    let x_yes = q_yes
        .checked_mul(FP_SCALE)
        .ok_or(MarketError::MathError)?
        .checked_div(b)
        .ok_or(MarketError::MathError)?;
    let x_no = q_no
        .checked_mul(FP_SCALE)
        .ok_or(MarketError::MathError)?
        .checked_div(b)
        .ok_or(MarketError::MathError)?;

    let exp_yes = exp_approx(x_yes)?;
    let exp_no = exp_approx(x_no)?;

    let sum = exp_yes.checked_add(exp_no).ok_or(MarketError::MathError)?;

    let ln_sum = ln_approx(sum)?;

    let cost = b
        .checked_mul(ln_sum)
        .ok_or(MarketError::MathError)?
        .checked_div(FP_SCALE)
        .ok_or(MarketError::MathError)?;

    Ok(cost)
}

pub fn exp_approx(x_fp: u128) -> Result<u128> {
    const XMAX_FP: u128 = 20u128 * FP_SCALE;

    let x = if x_fp > XMAX_FP { XMAX_FP } else { x_fp };

    let one = FP_SCALE;
    let x1 = x;

    let x2 = x1
        .checked_mul(x1)
        .ok_or(MarketError::MathError)?
        .checked_div(FP_SCALE)
        .ok_or(MarketError::MathError)?;
    let x2div2 = x2.checked_div(2).ok_or(MarketError::MathError)?;

    let x3 = x2
        .checked_mul(x1)
        .ok_or(MarketError::MathError)?
        .checked_div(FP_SCALE)
        .ok_or(MarketError::MathError)?;
    let x3div6 = x3.checked_div(6).ok_or(MarketError::MathError)?;

    let x4 = x3
        .checked_mul(x1)
        .ok_or(MarketError::MathError)?
        .checked_div(FP_SCALE)
        .ok_or(MarketError::MathError)?;
    let x4div24 = x4.checked_div(24).ok_or(MarketError::MathError)?;

    let x5 = x4
        .checked_mul(x1)
        .ok_or(MarketError::MathError)?
        .checked_div(FP_SCALE)
        .ok_or(MarketError::MathError)?;
    let x5div120 = x5.checked_div(120).ok_or(MarketError::MathError)?;

    let mut sum = one;
    sum = sum.checked_add(x1).ok_or(MarketError::MathError)?;
    sum = sum.checked_add(x2div2).ok_or(MarketError::MathError)?;
    sum = sum.checked_add(x3div6).ok_or(MarketError::MathError)?;
    sum = sum.checked_add(x4div24).ok_or(MarketError::MathError)?;
    sum = sum.checked_add(x5div120).ok_or(MarketError::MathError)?;

    Ok(sum)
}

pub fn ln_approx(y_fp: u128) -> Result<u128> {
    if y_fp < FP_SCALE {
        return err!(MarketError::MathError);
    }

    let numerator = y_fp.checked_sub(FP_SCALE).ok_or(MarketError::MathError)?;
    let denominator = y_fp.checked_sub(FP_SCALE).ok_or(MarketError::MathError)?;

    let z_fp = numerator
        .checked_mul(FP_SCALE)
        .ok_or(MarketError::MathError)?
        .checked_div(denominator)
        .ok_or(MarketError::MathError)?;

    let z1 = z_fp;
    let z2 = z1
        .checked_mul(z1)
        .ok_or(MarketError::MathError)?
        .checked_div(FP_SCALE)
        .ok_or(MarketError::MathError)?;
    let z3 = z2
        .checked_mul(z1)
        .ok_or(MarketError::MathError)?
        .checked_div(FP_SCALE)
        .ok_or(MarketError::MathError)?;
    let z5 = z3
        .checked_mul(z2)
        .ok_or(MarketError::MathError)?
        .checked_div(FP_SCALE)
        .ok_or(MarketError::MathError)?;

    let mut sum = z1;
    sum = sum
        .checked_add(z3.checked_div(3).ok_or(MarketError::MathError)?)
        .ok_or(MarketError::MathError)?;
    sum = sum
        .checked_add(z5.checked_div(5).ok_or(MarketError::MathError)?)
        .ok_or(MarketError::MathError)?;

    let ln_fp = sum.checked_mul(2).ok_or(MarketError::MathError)?;
    Ok(ln_fp)
}

pub fn amm_execute_buy(
    market_q_yes: &mut u128,
    market_q_no: &mut u128,
    b_fp: u128,
    outcome: u8,
    quantity: u128,
) -> Result<u128> {
    if quantity == 0 {
        return Ok(0);
    }

    let cost_fp: u128 = if outcome == OutcomeSide::Yes as u8 {
        amm_cost_to_yes(*market_q_yes, *market_q_no, b_fp, quantity)?
    } else {
        amm_cost_to_no(*market_q_yes, *market_q_no, b_fp, quantity)?
    };

    if outcome == OutcomeSide::Yes as u8 {
        *market_q_yes = market_q_yes
            .checked_add(quantity)
            .ok_or(MarketError::MathError)?;
    } else {
        *market_q_no = market_q_no
            .checked_add(quantity)
            .ok_or(MarketError::MathError)?;
    }

    Ok(cost_fp)
}

pub fn amm_price_per_token(q_yes: u128, q_no: u128, b: u128, outcome: u8) -> Result<u128> {
    if b == 0 {
        return err!(MarketError::MathError);
    }

    let x_yes = q_yes
        .checked_mul(FP_SCALE)
        .ok_or(MarketError::MathError)?
        .checked_div(b)
        .ok_or(MarketError::MathError)?;
    let x_no = q_no
        .checked_mul(FP_SCALE)
        .ok_or(MarketError::MathError)?
        .checked_div(b)
        .ok_or(MarketError::MathError)?;

    let exp_x = exp_approx(x_yes)?;
    let exp_y = exp_approx(x_no)?;

    let sum = exp_x.checked_add(exp_y).ok_or(MarketError::MathError)?;

    if sum == 0 {
        return err!(MarketError::MathError);
    }

    let prob_x = exp_x
        .checked_mul(FP_SCALE)
        .ok_or(MarketError::MathError)?
        .checked_div(sum)
        .ok_or(MarketError::MathError)?;
    let prob_y = FP_SCALE.checked_sub(prob_x).ok_or(MarketError::MathError)?;

    let price_per_token = if outcome == OutcomeSide::Yes as u8 {
        prob_x
    } else {
        prob_y
    };

    Ok(price_per_token)
}
