//! Template recommendations. Ready = every app in the template is connected. One-away = one
//! app missing (a connect prompt). Rank: outcome tag match, then installs.
use crate::util::*;
use serde_json::{json, Value};

pub fn recommend_op(i: &Value) -> R<Value> {
    let have: Vec<String> = arr(i, "connected_apps")?.iter().filter_map(Value::as_str).map(str::to_lowercase).collect();
    let goals: Vec<String> = i.get("goals").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(str::to_lowercase).collect();
    let mut ready = vec![];
    let mut one = vec![];
    for t in arr(i, "catalog")? {
        let apps: Vec<String> = arr(t, "apps")?.iter().filter_map(Value::as_str).map(str::to_lowercase).collect();
        let missing: Vec<&String> = apps.iter().filter(|a| !have.contains(a)).collect();
        let tags: Vec<String> = t.get("tags").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(str::to_lowercase).collect();
        let goal_hits = tags.iter().filter(|x| goals.contains(x)).count();
        let installs = f_or(t, "installs", 0.0);
        let score = goal_hits as f64 * 10.0 + (1.0 + installs).log10();
        let row = json!({"template": s(t, "name")?, "apps": apps, "missing": missing, "goal_match": goal_hits, "installs": installs, "score": rd(score, 2)});
        match missing.len() {
            0 => ready.push(row),
            1 => one.push(row),
            _ => {}
        }
    }
    let by = |a: &Value, b: &Value| b["score"].as_f64().unwrap().total_cmp(&a["score"].as_f64().unwrap());
    ready.sort_by(by);
    one.sort_by(by);
    let mut unlock: std::collections::BTreeMap<String, usize> = Default::default();
    for r in &one {
        *unlock.entry(r["missing"][0].as_str().unwrap().to_string()).or_default() += 1;
    }
    let best = unlock.iter().max_by_key(|x| x.1).map(|x| json!({"connect": x.0, "unlocks_templates": x.1}));
    Ok(json!({"ready_now": ready, "one_connection_away": one, "best_next_connection": best}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn ready_and_one_away() {
        let i = json!({"connected_apps":["Slack","HubSpot"],"goals":["routing"],"catalog":[
            {"name":"a","apps":["slack","hubspot"],"tags":["routing"],"installs":10},{"name":"b","apps":["slack","stripe"],"installs":1000},
            {"name":"c","apps":["stripe","gong"]}]});
        let r = recommend_op(&i).unwrap();
        assert_eq!(r["ready_now"][0]["template"], "a");
        assert_eq!(r["one_connection_away"][0]["template"], "b");
        assert_eq!(r["best_next_connection"]["connect"], "stripe");
    }
}
