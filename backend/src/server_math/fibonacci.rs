use crate::server_math::req_resp::{MathResponse, NumRequest, AppState, BasicResponse};
use crate::server_math::store::Store;
use actix_web::{get, web, Responder, Result, HttpResponse};
use num_bigint::BigInt;
use num_traits::{Zero, One};

fn find_nth_fibonacci<S: Store>(num: i64, store: &mut S) -> Result<(BigInt, bool), String> {
    if num < 0 {
        return Err("Number must be non-negative".to_string());
    }

    if num <= 1 {
        return if num == 0 {
            Ok((BigInt::zero(), true))
        } else {
            Ok((BigInt::one(), true))
        };
    }

    // Check if result is in cache
    if store.contains_key(num)? {
        if let Some(result) = store.get(num)? {
            return Ok((result, true));
        }
    }

    // Find the largest calculated Fibonacci number in our store
    let mut max_calculated = 1;
    for i in 0..=num {
        if store.contains_key(i)? {
            max_calculated = i;
        } else {
            break;
        }
    }

    // Get the last two calculated numbers

    let mut current = store.get(max_calculated)?.unwrap_or_else(BigInt::one);
    let mut prev = store.get(max_calculated - 1)?.unwrap_or_else(BigInt::zero);

    // Calculate remaining numbers iteratively
    for n in (max_calculated + 1)..=num {
        let next = current.clone() + prev.clone();
        store.set(n, &next)?;
        prev = current;
        current = next;
    }

    store.get(num)?
        .map(|result| (result, false))
        .ok_or_else(|| "Failed to retrieve calculated result".to_string())
}

#[get("/fib")]
async fn calc_fib(
    fib_request: web::Query<NumRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder> {
    let num = fib_request.num.unwrap_or(0);
    let mut store = data.fib_store.lock().unwrap();

    match find_nth_fibonacci(num, &mut *store) {
        Ok((result, was_cached)) => {
            let message = if was_cached {
                format!("Fibonacci number {} retrieved from cache", num)
            } else {
                format!("Fibonacci number {} calculated", num)
            };

            Ok(HttpResponse::Ok().json(MathResponse {
                message,
                result: result.to_string(),
                cached: was_cached,
            }))
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(BasicResponse { message: e })),
    }
}

