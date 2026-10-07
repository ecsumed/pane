use ratatui::layout::Rect;

use super::node::PaneKey;
use super::node_data::CardinalDirection;

struct Candidate {
    key: PaneKey,
    gap: i32,
    overlap: i32,
    offset: i32,
}

fn span_overlap(a_start: u16, a_end: u16, b_start: u16, b_end: u16) -> i32 {
    a_end.min(b_end) as i32 - a_start.max(b_start) as i32
}

fn candidate(active: Rect, other: Rect, direction: CardinalDirection) -> Option<(i32, i32, i32)> {
    let (gap, overlap, offset) = match direction {
        CardinalDirection::Right if other.x > active.x => (
            other.left() as i32 - active.right() as i32,
            span_overlap(active.top(), active.bottom(), other.top(), other.bottom()),
            (other.y as i32 - active.y as i32).abs(),
        ),
        CardinalDirection::Left if other.x < active.x => (
            active.left() as i32 - other.right() as i32,
            span_overlap(active.top(), active.bottom(), other.top(), other.bottom()),
            (other.y as i32 - active.y as i32).abs(),
        ),
        CardinalDirection::Down if other.y > active.y => (
            other.top() as i32 - active.bottom() as i32,
            span_overlap(active.left(), active.right(), other.left(), other.right()),
            (other.x as i32 - active.x as i32).abs(),
        ),
        CardinalDirection::Up if other.y < active.y => (
            active.top() as i32 - other.bottom() as i32,
            span_overlap(active.left(), active.right(), other.left(), other.right()),
            (other.x as i32 - active.x as i32).abs(),
        ),
        _ => return None,
    };
    (gap >= -1 && overlap > 0).then_some((gap, overlap, offset))
}

pub fn neighbour(
    rects: &[(PaneKey, Rect)],
    active: PaneKey,
    direction: CardinalDirection,
    previous: Option<PaneKey>,
) -> Option<PaneKey> {
    let &(_, active_rect) = rects.iter().find(|(key, _)| *key == active)?;

    let candidates: Vec<Candidate> = rects
        .iter()
        .filter(|(key, _)| *key != active)
        .filter_map(|&(key, rect)| {
            let (gap, overlap, offset) = candidate(active_rect, rect, direction)?;
            Some(Candidate {
                key,
                gap,
                overlap,
                offset,
            })
        })
        .collect();

    let nearest = candidates.iter().map(|c| c.gap).min()?;
    let adjacent = candidates.iter().filter(|c| c.gap == nearest);

    if let Some(previous) = adjacent.clone().find(|c| Some(c.key) == previous) {
        return Some(previous.key);
    }
    adjacent
        .max_by(|a, b| a.overlap.cmp(&b.overlap).then(b.offset.cmp(&a.offset)))
        .map(|c| c.key)
}

#[cfg(test)]
mod tests {
    use slotmap::SlotMap;

    use super::*;

    fn keys(n: usize) -> Vec<PaneKey> {
        let mut map: SlotMap<PaneKey, ()> = SlotMap::with_key();
        (0..n).map(|_| map.insert(())).collect()
    }

    #[test]
    fn test_moves_to_adjacent_pane_not_nearest_centre() {
        let k = keys(3);
        let rects = [
            (k[0], Rect::new(0, 0, 50, 40)),
            (k[1], Rect::new(50, 0, 50, 10)),
            (k[2], Rect::new(50, 10, 50, 30)),
        ];
        assert_eq!(
            neighbour(&rects, k[1], CardinalDirection::Left, None),
            Some(k[0])
        );
        assert_eq!(
            neighbour(&rects, k[1], CardinalDirection::Down, None),
            Some(k[2])
        );
        assert_eq!(
            neighbour(&rects, k[0], CardinalDirection::Right, None),
            Some(k[2])
        );
        assert_eq!(neighbour(&rects, k[0], CardinalDirection::Left, None), None);
        assert_eq!(neighbour(&rects, k[1], CardinalDirection::Up, None), None);
    }

    #[test]
    fn test_prefers_the_pane_you_came_from() {
        let k = keys(3);
        let rects = [
            (k[0], Rect::new(0, 0, 50, 40)),
            (k[1], Rect::new(50, 0, 50, 20)),
            (k[2], Rect::new(50, 20, 50, 20)),
        ];
        assert_eq!(
            neighbour(&rects, k[0], CardinalDirection::Right, None),
            Some(k[1])
        );
        assert_eq!(
            neighbour(&rects, k[0], CardinalDirection::Right, Some(k[2])),
            Some(k[2])
        );
    }

    #[test]
    fn test_handles_collapsed_borders_that_overlap_by_one_cell() {
        let k = keys(2);
        let rects = [
            (k[0], Rect::new(0, 0, 51, 40)),
            (k[1], Rect::new(50, 0, 50, 40)),
        ];
        assert_eq!(
            neighbour(&rects, k[0], CardinalDirection::Right, None),
            Some(k[1])
        );
        assert_eq!(
            neighbour(&rects, k[1], CardinalDirection::Left, None),
            Some(k[0])
        );
    }
}
