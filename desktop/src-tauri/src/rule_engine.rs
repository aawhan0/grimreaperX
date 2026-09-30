use crate::models::Tier;

pub fn evaluate(duration_minutes: i64, threshold_minutes: i64) -> Tier {
    let over = duration_minutes - threshold_minutes;
    if over < 0 { Tier::Normal }
    else if over < 5 { Tier::Mild }
    else if over < 30 { Tier::Serious }
    else { Tier::Unhinged }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn below_threshold() { assert_eq!(evaluate(19, 20), Tier::Normal); }
    #[test] fn threshold_crossed() { assert_eq!(evaluate(20, 20), Tier::Mild); }
    #[test] fn serious() { assert_eq!(evaluate(25, 20), Tier::Serious); }
    #[test] fn unhinged() { assert_eq!(evaluate(50, 20), Tier::Unhinged); }
}
