const METER_FILLED: &str = "▮";
const METER_EMPTY: &str = "▯";

pub fn generate_history_meter_string(current: usize, capacity: usize) -> String {
    let current = current.min(capacity);

    let (nx, ny) = if capacity > 10 {
        ((current * 10) / capacity, 10)
    } else {
        (current, capacity)
    };

    let nz = ny - nx;
    format!("{}{}", METER_FILLED.repeat(nx), METER_EMPTY.repeat(nz))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meter_standard_scaling() {
        let result = generate_history_meter_string(3, 5);
        assert_eq!(result, "▮▮▮▯▯");
        assert_eq!(result.chars().count(), 5);
    }

    #[test]
    fn test_meter_normalization_over_ten() {
        // Given x=20, y=100
        // Scaled: nx = (20 * 10) / 100 = 2, ny = 10
        // Result should be 2 'x's and 8 'z's
        let result = generate_history_meter_string(20, 100);
        assert_eq!(result, "▮▮▯▯▯▯▯▯▯▯");
        assert_eq!(result.chars().count(), 10);
    }

    #[test]
    fn test_meter_rounding_accuracy() {
        // Test near-half cases: x=7, y=15
        // nx = (7 * 10) / 15 = 4.66...
        let result = generate_history_meter_string(7, 15);
        assert_eq!(result.chars().count(), 10);
        assert!(result.starts_with("▮▮▮▮"));
    }

    #[test]
    fn test_meter_at_capacity() {
        // x=10, y=10 -> "▮▮▮▮▮▮▮▮▮▮"
        let result = generate_history_meter_string(10, 10);
        assert_eq!(result, "▮▮▮▮▮▮▮▮▮▮");
    }

    #[test]
    fn test_meter_zero_difference() {
        // x=5, y=5 -> "▮▮▮▮▮" (0 'z's)
        let result = generate_history_meter_string(5, 5);
        assert_eq!(result, "▮▮▮▮▮");
    }
}
