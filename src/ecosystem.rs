//! Ecosystem revenue. Sourced = the partner brought the deal. Influenced = a partner touched a
//! deal it did not source. Integration lift = win rate of closed deals with an integration in
//! use minus win rate without.
use crate::util::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn revenue_op(i: &Value) -> R<Value> {
    let mut per: BTreeMap<String, (f64, f64, usize)> = BTreeMap::new();
    let (mut tot_won, mut src, mut inf) = (0.0, 0.0, 0.0);
    let (mut wi, mut ni, mut wo, mut no) = (0, 0, 0, 0);
    for d in arr(i, "deals")? {
        let amt = f(d, "amount")?;
        let st = s(d, "stage")?;
        let won = st == "won";
        if !["won", "lost", "open"].contains(&st) {
            return Err(format!("stage must be won, lost or open, got `{st}`"));
        }
        let sourced_by = d.get("sourced_by").and_then(Value::as_str);
        let touched: Vec<&str> = d.get("partners_involved").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
        let integ = d.get("integrations_used").and_then(Value::as_array).map_or(false, |a| !a.is_empty());
        if st != "open" {
            if integ {
                ni += 1;
                wi += won as usize;
            } else {
                no += 1;
                wo += won as usize;
            }
        }
        if !won {
            continue;
        }
        tot_won += amt;
        if let Some(p) = sourced_by {
            src += amt;
            let e = per.entry(p.into()).or_default();
            e.0 += amt;
            e.2 += 1;
        }
        let infl: Vec<&&str> = touched.iter().filter(|p| Some(**p) != sourced_by).collect();
        if sourced_by.is_none() && !infl.is_empty() {
            inf += amt;
        }
        for p in infl {
            let e = per.entry(p.to_string()).or_default();
            e.1 += amt;
            e.2 += 1;
        }
    }
    let rate = |w: usize, n: usize| if n > 0 { Some(rd(w as f64 / n as f64 * 100.0, 1)) } else { None };
    let (ri, ro) = (rate(wi, ni), rate(wo, no));
    let table: Vec<Value> = per.into_iter().map(|(p, (s_, i_, n))| json!({"partner": p, "sourced": s_, "influenced": i_, "won_deals": n})).collect();
    Ok(json!({"won_revenue": tot_won, "partner_sourced": src, "partner_influenced_only": inf,
        "partner_share_pct": rd((src + inf) / tot_won.max(1e-9) * 100.0, 1),
        "win_rate_with_integration_pct": ri, "win_rate_without_pct": ro,
        "integration_lift_pts": ri.zip(ro).map(|(a, b)| rd(a - b, 1)), "closed_deals": {"with_integration": ni, "without": no},
        "partners": table}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn sourced_influenced_lift() {
        let i = json!({"deals":[{"amount":100,"stage":"won","sourced_by":"P","integrations_used":["x"]},
            {"amount":50,"stage":"won","partners_involved":["Q"]},{"amount":10,"stage":"lost","integrations_used":["x"]},{"amount":5,"stage":"lost"}]});
        let r = revenue_op(&i).unwrap();
        assert_eq!((r["partner_sourced"].as_f64(), r["partner_influenced_only"].as_f64()), (Some(100.0), Some(50.0)));
        assert_eq!(r["integration_lift_pts"], 0.0);
    }
}
